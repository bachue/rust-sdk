use super::{
    super::bucket::Bucket, object_uploader::ResumablePolicy, CreateUploaderError, CreateUploaderResult, UploadManager,
    UploadPolicy, UploadResult, UploadToken,
};
use crate::{utils::ron::Ron, Config, Credential};
use mime::Mime;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{Read, Result},
    mem::replace,
    path::Path,
    sync::Arc,
};

type OnUploadingProgressCallback = Box<dyn Fn(u64, Option<u64>) + Send + Sync>;
type OnCompletedCallback = Box<dyn Fn(UploadResult) + Send + Sync>;

enum BatchUploadTarget {
    File(File),
    Stream(Box<dyn Read + Send>),
}

/// 批量上传任务，包装一个上传任务供批量上传器负责上传
#[must_use = "创建上传任务并不会真正上传文件，您需要将当前任务提交到批量上传器后，调用 `start` 方法执行上传任务"]
pub struct BatchUploadJob {
    key: Option<String>,
    upload_token: Option<UploadToken>,
    vars: HashMap<String, String>,
    metadata: HashMap<String, String>,
    checksum_enabled: bool,
    resumable_policy: Option<ResumablePolicy>,
    file_name: String,
    mime: Option<Mime>,
    on_uploading_progress: Option<OnUploadingProgressCallback>,
    on_completed: Option<OnCompletedCallback>,
    target: BatchUploadTarget,
    expected_data_size: u64,
}

/// 批量上传任务生成器，提供上传数据所需的多个参数
pub struct BatchUploadJobBuilder {
    key: Option<String>,
    upload_token: Option<UploadToken>,
    vars: HashMap<String, String>,
    metadata: HashMap<String, String>,
    checksum_enabled: bool,
    on_uploading_progress: Option<OnUploadingProgressCallback>,
    on_completed: Option<OnCompletedCallback>,
    resumable_policy: Option<ResumablePolicy>,
}

enum BatchUploaderCore {
    UploadManager {
        upload_manager: UploadManager,
        upload_token: UploadToken,
    },
    Bucket(Bucket),
}

struct BatchUploaderContext {
    core: BatchUploaderCore,
    max_concurrency: usize,
    thread_pool_size: usize,
}

/// 批量上传器，上传之前所有提交的任务
pub struct BatchUploader {
    context: BatchUploaderContext,
    jobs: Vec<BatchUploadJob>,
}

impl BatchUploader {
    pub(super) fn new_for_upload_manager(
        upload_manager: UploadManager,
        upload_token: UploadToken,
    ) -> CreateUploaderResult<Self> {
        if upload_token.policy()?.bucket().is_none() {
            return Err(CreateUploaderError::BucketIsMissingInUploadToken);
        }
        Ok(Self {
            jobs: Vec::new(),
            context: BatchUploaderContext {
                core: BatchUploaderCore::UploadManager {
                    upload_manager,
                    upload_token,
                },
                max_concurrency: 0,
                thread_pool_size: 0,
            },
        })
    }

    pub(in super::super) fn new_for_bucket(bucket: Bucket) -> Self {
        Self {
            jobs: Vec::new(),
            context: BatchUploaderContext {
                core: BatchUploaderCore::Bucket(bucket),
                max_concurrency: 0,
                thread_pool_size: 0,
            },
        }
    }

    /// 预期批量上传的任务数量
    pub fn expected_jobs_count(&mut self, expected_jobs_count: usize) -> &mut Self {
        if expected_jobs_count > self.jobs.len() {
            self.jobs.reserve(expected_jobs_count - self.jobs.len());
        }
        self
    }

    /// 为上传器创建专用线程池指定线程池大小
    ///
    /// 批量上传器总是优先使用上传管理器中的线程池，如果上传管理器中没有创建过线程池，则自行创建专用线程池
    pub fn thread_pool_size(&mut self, num_threads: usize) -> &mut Self {
        self.context.thread_pool_size = num_threads;
        self
    }

    /// 上传文件最大并发度
    ///
    /// 默认情况下，上传文件时的最大并发度等于其使用的线程池大小。
    /// 调用该方法可以修改最大并发度
    pub fn max_concurrency(&mut self, concurrency: usize) -> &mut Self {
        self.context.max_concurrency = concurrency;
        self
    }

    /// 提交上传任务
    pub fn push_job(&mut self, job: BatchUploadJob) -> &mut Self {
        self.jobs.push(job);
        self
    }

    /// 开始执行上传任务
    ///
    /// 需要注意的是，该方法会持续阻塞直到上传任务全部执行完毕（不保证执行顺序）。
    /// 该方法不返回任何结果，上传结果由每个上传任务内定义的 `on_completed` 回调负责返回。
    ///
    /// 方法返回后，当前批量上传器的上传任务将被清空，但其他参数都将保留，可以重新添加任务并复用
    pub fn start(&mut self) {
        let thread_pool = build_thread_pool(&self.context);
        let context = &self.context;
        let mut jobs = replace(&mut self.jobs, Vec::new());

        thread_pool.scope(|s| {
            while let Some(job) = jobs.pop() {
                s.spawn(|_| handle_job(context, job, &thread_pool))
            }
        });

        self.jobs = jobs;
    }
}

/// 构建线程池
///
/// 默认情况下总是使用上传管理器的线程池。
/// 如果没有或该线程池尺寸只有 1，则自行创建。
/// 自行创建时将会使用 `thread_pool_size` 的建议，如果没有建议，就使用 CPU 数量（但如果 CPU 的数量为 1，则使用 2）。
/// 确保返回的线程池尺寸必须大于 1，否则可能会导致死锁
fn build_thread_pool(context: &BatchUploaderContext) -> Ron<'_, ThreadPool> {
    context
        .core
        .thread_pool()
        .filter(|pool| pool.current_num_threads() > 1)
        .map(|pool| Ron::Referenced(pool.as_ref()))
        .unwrap_or_else(|| {
            let mut builder = ThreadPoolBuilder::new();
            if context.thread_pool_size > 0 {
                builder = builder.num_threads(context.thread_pool_size);
            }
            Ron::Owned(
                builder
                    .thread_name(|index| format!("qiniu_ng_batch_uploader_worker_{}", index))
                    .build()
                    .unwrap(),
            )
        })
}

fn handle_job(context: &BatchUploaderContext, job: BatchUploadJob, thread_pool: &ThreadPool) {
    let BatchUploadJob {
        key,
        upload_token,
        vars,
        metadata,
        checksum_enabled,
        resumable_policy,
        file_name,
        mime,
        target,
        expected_data_size,
        on_uploading_progress,
        on_completed,
    } = job;

    let mut object_uploader = match &context.core {
        BatchUploaderCore::UploadManager {
            upload_manager,
            upload_token: context_upload_token,
        } => upload_manager
            .upload_for_upload_token(
                upload_token
                    .map(Cow::Owned)
                    .unwrap_or_else(|| Cow::Borrowed(context_upload_token)),
            )
            .unwrap(),
        BatchUploaderCore::Bucket(bucket) => bucket.uploader(),
    };
    object_uploader = object_uploader
        .thread_pool(thread_pool)
        .max_concurrency(context.max_concurrency);
    if let Some(key) = key {
        object_uploader = object_uploader.key(key);
    }
    for (var_name, var_value) in vars.into_iter() {
        object_uploader = object_uploader.var(var_name, var_value);
    }
    for (metadata_name, metadata_value) in metadata.into_iter() {
        object_uploader = object_uploader.metadata(metadata_name, metadata_value);
    }
    if checksum_enabled {
        object_uploader = object_uploader.enable_checksum();
    } else {
        object_uploader = object_uploader.disable_checksum();
    }
    if let Some(on_uploading_progress) = on_uploading_progress {
        object_uploader = object_uploader.on_progress(on_uploading_progress);
    }
    if let Some(resumable_policy) = resumable_policy {
        match resumable_policy {
            ResumablePolicy::Threshold(threshold) => {
                object_uploader = object_uploader.upload_threshold(threshold);
            }
            ResumablePolicy::Never => {
                object_uploader = object_uploader.never_be_resumable();
            }
            ResumablePolicy::Always => {
                object_uploader = object_uploader.always_be_resumable();
            }
        }
    }
    let upload_result = match target {
        BatchUploadTarget::File(file) => object_uploader.upload_stream(file, expected_data_size, file_name, mime),
        BatchUploadTarget::Stream(reader) => object_uploader.upload_stream(reader, expected_data_size, file_name, mime),
    };
    if let Some(on_completed) = on_completed.as_ref() {
        on_completed(upload_result);
    }
}

impl Default for BatchUploadJobBuilder {
    fn default() -> Self {
        Self {
            key: None,
            upload_token: None,
            vars: HashMap::new(),
            metadata: HashMap::new(),
            checksum_enabled: true,
            on_uploading_progress: None,
            on_completed: None,
            resumable_policy: None,
        }
    }
}

impl BatchUploadJobBuilder {
    /// 指定上传对象的名称
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// 指定上传所用的上传凭证
    ///
    /// 默认情况下，总是复用批量上传器创建时传入的上传凭证。
    /// 该方法则可以在指定上传当前对象时使用上传凭证
    pub fn upload_token(mut self, upload_token: impl Into<UploadToken>) -> CreateUploaderResult<Self> {
        let upload_token = upload_token.into();
        if upload_token.policy()?.bucket().is_none() {
            return Err(CreateUploaderError::BucketIsMissingInUploadToken);
        }
        self.upload_token = Some(upload_token);
        Ok(self)
    }

    /// 指定上传所用的上传策略
    ///
    /// 默认情况下，总是复用批量上传器创建时传入的上传凭证。
    /// 该方法则可以在指定上传当前对象时使用上传策略生成的上传凭证
    pub fn upload_policy(
        self,
        upload_policy: UploadPolicy,
        credential: impl Into<Credential>,
    ) -> CreateUploaderResult<Self> {
        self.upload_token(UploadToken::new(upload_policy, credential.into()))
    }

    /// 指定上传所用的存储空间和认证信息
    ///
    /// 默认情况下，总是复用批量上传器创建时传入的上传凭证。
    /// 该方法则可以在指定上传当前对象时使用的根据存储空间和认证信息生成的上传凭证
    pub fn upload_for_bucket(
        self,
        bucket: impl Into<Cow<'static, str>>,
        credential: Credential,
        config: &Config,
    ) -> Self {
        self.upload_token(UploadToken::new_from_bucket(bucket.into(), credential, config))
            .unwrap()
    }

    /// 为上传对象指定[自定义变量](https://developer.qiniu.com/kodo/manual/1235/vars#xvar)
    ///
    /// 可以多次调用以指定多个自定义变量
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// 为上传对象指定自定义元数据
    ///
    /// 可以多次调用以指定多个自定义元数据
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 禁用上传数据校验
    ///
    /// 在任何场景下都不推荐禁用上传数据校验
    pub fn disable_checksum(mut self) -> Self {
        self.checksum_enabled = false;
        self
    }

    /// 启用上传数据校验
    ///
    /// 默认总是启用，在任何场景下都不推荐禁用上传数据校验
    pub fn enable_checksum(mut self) -> Self {
        self.checksum_enabled = true;
        self
    }

    /// 指定分片上传策略阙值
    ///
    /// 对于上传文件的情况，如果文件尺寸大于该值，将自动使用分片上传，否则，使用表单上传。
    /// 单位为字节，默认将采用客户端配置中的配置项。
    ///
    /// 对于上传数据流的情况，由于无法预知数据尺寸，将总是使用分片上传
    pub fn upload_threshold(mut self, threshold: u32) -> Self {
        self.resumable_policy = Some(ResumablePolicy::Threshold(threshold));
        self
    }

    /// 总是使用分片上传
    pub fn always_be_resumable(mut self) -> Self {
        self.resumable_policy = Some(ResumablePolicy::Always);
        self
    }

    /// 总是使用表单上传
    ///
    /// 需要注意的是，虽然表单上传仅需要一次 HTTP 调用，性能优于分片上传，
    /// 但分片上传具有断点续传的特性，以及表单上传会将整个文件内容都加载进内存中，对大文件极不友好。
    /// 因此总是推荐使用默认策略，如果认为默认阙值过小，可以适当提高客户端配置的阙值。
    pub fn never_be_resumable(mut self) -> Self {
        self.resumable_policy = Some(ResumablePolicy::Never);
        self
    }

    /// 上传进度回调
    ///
    /// 将在上传期间反复回调指定的闭包，以获取上传进度。
    /// 上传进度闭包的第一个参数为已经上传的数据量，
    /// 第二个参数为数据总量，如果为 `None` 表示数据总量不可预知，
    /// 单位均为字节
    pub fn on_progress(mut self, progress: impl Fn(u64, Option<u64>) + Send + Sync + 'static) -> Self {
        self.on_uploading_progress = Some(Box::new(progress));
        self
    }

    /// 完成上传回调
    ///
    /// 将在上传完毕后回调指定的闭包，返回上传结果。
    pub fn on_completed(mut self, on_completed: impl Fn(UploadResult) + Send + Sync + 'static) -> Self {
        self.on_completed = Some(Box::new(on_completed));
        self
    }

    /// 上传文件
    ///
    /// 该方法用于生成批量上传任务，用于上传指定路径的文件
    pub fn upload_file(
        self,
        file_path: impl AsRef<Path>,
        file_name: impl Into<String>,
        mime: Option<Mime>,
    ) -> Result<BatchUploadJob> {
        let file = File::open(file_path.as_ref())?;
        let job = BatchUploadJob {
            key: self.key,
            upload_token: self.upload_token,
            vars: self.vars,
            metadata: self.metadata,
            checksum_enabled: self.checksum_enabled,
            resumable_policy: self.resumable_policy,
            on_uploading_progress: self.on_uploading_progress,
            on_completed: self.on_completed,
            file_name: file_name.into(),
            mime,
            expected_data_size: file.metadata()?.len(),
            target: BatchUploadTarget::File(file),
        };
        Ok(job)
    }

    /// 上传数据流
    ///
    /// 该方法用于生成批量上传任务，用于上传指定的数据流
    pub fn upload_stream(
        self,
        stream: impl Read + Send + 'static,
        size: u64,
        file_name: impl Into<String>,
        mime: Option<Mime>,
    ) -> BatchUploadJob {
        BatchUploadJob {
            key: self.key,
            upload_token: self.upload_token,
            vars: self.vars,
            metadata: self.metadata,
            checksum_enabled: self.checksum_enabled,
            resumable_policy: self.resumable_policy,
            on_uploading_progress: self.on_uploading_progress,
            on_completed: self.on_completed,
            file_name: file_name.into(),
            mime,
            expected_data_size: size,
            target: BatchUploadTarget::Stream(Box::new(stream)),
        }
    }
}

impl BatchUploaderCore {
    fn thread_pool(&self) -> Option<&Arc<ThreadPool>> {
        match self {
            Self::UploadManager { upload_manager, .. } => upload_manager.thread_pool(),
            Self::Bucket(bucket) => bucket.thread_pool(),
        }
    }
}
