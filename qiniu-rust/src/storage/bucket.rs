//! 存储空间模块

use super::{
    object::Object,
    region::{Region, RegionId},
    uploader::{BatchUploader, ObjectUploader, UploadManager, UploadToken},
};
use crate::{
    config::Config,
    credential::Credential,
    http::{Client, Error as HTTPError, Result as HTTPResult, TokenVersion},
};
use assert_impl::assert_impl;
use once_cell::sync::OnceCell;
use rayon::ThreadPool;
use serde::Deserialize;
use std::{
    borrow::{Borrow, Cow},
    ffi::c_void,
    iter::Iterator,
    result::Result,
    sync::Arc,
};
use thiserror::Error;
use url::{ParseError as UrlParseError, Url};

/// 存储空间
///
/// 封装存储空间相关数据，例如配置，区域，下载域名等
#[derive(Clone)]
pub struct Bucket(Arc<BucketInner>);

struct BucketInner {
    name: Cow<'static, str>,
    credential: Credential,
    upload_manager: UploadManager,
    region: OnceCell<Cow<'static, Region>>,
    backup_regions: OnceCell<Box<[Cow<'static, Region>]>>,
    domains: OnceCell<Box<[Cow<'static, str>]>>,
    rs_urls: OnceCell<Box<[String]>>,
    http_client: Client,
    bucket_info: OnceCell<BucketInfo>,
}

/// 存储空间生成器
///
/// 注意，该结构体仅用于在 SDK 中配置生成存储空间实例，而非在七牛云服务器上创建新的存储空间。
/// 事实上，除非您使用了私有云，或七牛以外的 CDN 服务商，否则您总是可以直接构建存储空间，存储空间为以懒加载的方式从七牛服务器获取区域信息和下载域名，SDK 确保懒加载的线程安全。
///
/// ```rust,no_run
/// use qiniu_ng::{Client, Config};
/// # use std::{result::Result, error::Error};
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let client = Client::new("[Access Key]", "[Secret Key]", Config::default());
/// let bucket = client.storage().bucket("[Bucket name]");
/// # Ok(())
/// # }
/// ```
pub struct BucketBuilder {
    name: Cow<'static, str>,
    credential: Credential,
    upload_manager: UploadManager,
    region: Option<Cow<'static, Region>>,
    backup_regions: Vec<Cow<'static, Region>>,
    domains: Vec<Cow<'static, str>>,
    http_client: Client,
}

/// 存储空间区域迭代器
pub struct BucketRegionIter<'a> {
    bucket: &'a Bucket,
    itered: usize,
}

#[derive(Deserialize)]
struct BucketInfo {
    private: u8,
}

impl BucketBuilder {
    pub(crate) fn new(name: Cow<'static, str>, credential: Credential, upload_manager: UploadManager) -> BucketBuilder {
        BucketBuilder {
            name,
            credential,
            http_client: Client::new(upload_manager.config().clone()),
            upload_manager,
            region: None,
            backup_regions: Vec::new(),
            domains: Vec::new(),
        }
    }

    /// 指定存储空间区域
    ///
    /// 注意：对于之前尚未指定过存储空间区域的情况，该方法将为存储空间指定区域。
    /// 而一旦指定过，之后调用该方法则表示指定备用区域。
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use qiniu_ng::{Client, Config, storage::region::Region};
    /// # use std::{result::Result, error::Error};
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let client = Client::new("[Access Key]", "[Secret Key]", Config::default());
    /// # let (region1, region2, region3) = (Region::z0(),Region::z1(),Region::z2());
    /// // 这里 bucket 将优先使用 `region1` 作为主要区域，而 `region2` 和 `region3` 则作为备用区域
    /// let bucket = client.storage().bucket_builder("[Bucket name]")
    ///                              .region(region1)
    ///                              .region(region2)
    ///                              .region(region3)
    ///                              .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn region(&mut self, region: impl Into<Cow<'static, Region>>) -> &mut Self {
        if self.region.is_none() {
            self.region = Some(region.into());
        } else {
            self.backup_regions.push(region.into());
        }
        self
    }

    /// 指定存储空间 ID
    ///
    /// 该方法仅适用于指定七牛公有云区域。
    /// 如果使用的是私有云，则请调用 `region` 方法。
    pub fn region_id(&mut self, region_id: RegionId) -> &mut Self {
        self.region(Cow::Borrowed(region_id.as_region()))
    }

    /// 自动检测区域
    ///
    /// 将连接七牛服务器查询当前存储空间所在区域和备用区域
    ///
    /// 注意，如果调用了该方法，则不应该再调用 `region` 或 `region_id` 方法。
    /// 除非有特殊需求，否则不建议您调用该方法，而是尽量使用懒加载的方式在必要时自动检测区域
    pub fn auto_detect_region(&mut self) -> HTTPResult<&mut Self> {
        let mut regions: Vec<Region> = Region::query(
            self.name.as_ref(),
            self.credential.access_key(),
            self.upload_manager.config().clone(),
        )?
        .into();
        self.region = Some(Cow::Owned(regions.swap_remove(0)));
        if !regions.is_empty() {
            self.backup_regions = regions.into_iter().map(Cow::Owned).collect();
        }
        Ok(self)
    }

    /// 新增下载域名
    ///
    /// 注意，可以先调用 `auto_detect_domains` 方法然后再调用该方法，SDK 将优先使用最后新增的域名
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use qiniu_ng::{Client, Config, storage::region::Region};
    /// # use std::{result::Result, error::Error};
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let client = Client::new("[Access Key]", "[Secret Key]", Config::default());
    ///
    /// // 这里 bucket 将优先使用 `cdn2.example.com` 作为下载域名，其次是 `cdn1.example.com`，最终才轮到七牛配置的下载域名
    /// let bucket = client.storage().bucket_builder("[Bucket name]")
    ///                              .auto_detect_domains()?
    ///                              .prepend_domain("cdn1.example.com")?
    ///                              .prepend_domain("cdn2.example.com")?
    ///                              .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn prepend_domain(&mut self, domain: impl Into<Cow<'static, str>>) -> Result<&mut Self, UrlParseError> {
        let domain = domain.into();
        let _ = Url::parse(&("http://".to_owned() + &domain))?;
        self.domains.push(domain);
        Ok(self)
    }

    /// 自动检测下载域名
    ///
    /// 将连接七牛服务器查询当前存储空间的下载域名列表
    pub fn auto_detect_domains(&mut self) -> HTTPResult<&mut Self> {
        self.domains = domain::query(&self.http_client, &self.credential, self.name.as_ref())?
            .into_iter()
            .map(Cow::Owned)
            .collect();
        Ok(self)
    }

    /// 生成存储空间
    ///
    /// 注意，该方法仅用于在 SDK 中配置生成存储空间实例，而非在七牛云服务器上创建新的存储空间
    pub fn build(&self) -> Bucket {
        let BucketBuilder {
            name,
            credential,
            upload_manager,
            http_client,
            region: original_region,
            backup_regions: original_backup_regions,
            domains: original_domains,
        } = self;

        let backup_regions = OnceCell::new();
        let region = original_region
            .to_owned()
            .map(|r| {
                backup_regions.get_or_init(|| original_backup_regions.to_owned().into_boxed_slice());
                OnceCell::from(r)
            })
            .unwrap_or_else(OnceCell::new);
        let domains = if original_domains.is_empty() {
            OnceCell::new()
        } else {
            let mut domains = original_domains.to_owned();
            domains.reverse();
            OnceCell::from(domains.into_boxed_slice())
        };
        Bucket(Arc::new(BucketInner {
            name: name.to_owned(),
            credential: credential.to_owned(),
            upload_manager: upload_manager.to_owned(),
            http_client: http_client.to_owned(),
            region,
            backup_regions,
            domains,
            rs_urls: OnceCell::new(),
            bucket_info: OnceCell::new(),
        }))
    }

    /// 重置生成器
    ///
    /// 重置生成器使得生成器可以被多次复用
    pub fn reset(&mut self, name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.name = name.into();
        self.region = None;
        self.backup_regions.clear();
        self.domains.clear();
        self
    }
}

impl Bucket {
    /// 存储空间名称
    pub fn name(&self) -> &str {
        self.0.name.as_ref()
    }

    /// 获取存储空间中的一个对象
    pub fn object(&self, key: impl Into<Cow<'static, str>>) -> Object {
        Object::new(self.to_owned(), key.into())
    }

    /// 存储空间区域
    ///
    /// 如果区域在存储空间生成前未指定，则该方法可能会连接七牛服务器查询当前存储空间所在区域
    pub fn region(&self) -> HTTPResult<&Region> {
        self.0
            .region
            .get_or_try_init(|| {
                let mut regions: Vec<Region> =
                    Region::query(self.name(), self.credential().access_key(), self.config().clone())?.into();
                let first_region = Cow::Owned(regions.swap_remove(0));
                self.0
                    .backup_regions
                    .get_or_init(|| regions.into_iter().map(Cow::Owned).collect());
                Ok(first_region)
            })
            .map(|region| region.as_ref())
    }

    /// 存储空间区域迭代器
    ///
    /// 该迭代器将首先返回当前存储空间所在区域，随后返回所有备用区域
    ///
    /// 如果区域在存储空间生成前未指定，则该方法可能会连接七牛服务器查询当前存储空间所在区域和备用区域
    pub fn regions<'a>(&'a self) -> HTTPResult<BucketRegionIter<'a>> {
        self.region()?;
        Ok(BucketRegionIter {
            bucket: self,
            itered: 0,
        })
    }

    /// 存储空间下载域名列表
    ///
    /// 如果下载域名在存储空间生成前未指定，则该方法可能会连接七牛服务器查询当前存储空间下载域名列表
    pub fn domains(&self) -> HTTPResult<Vec<&str>> {
        let domains = self.0.domains.get_or_try_init(|| {
            Ok(domain::query(&self.0.http_client, &self.0.credential, self.name())?
                .into_iter()
                .map(Cow::Owned)
                .collect())
        })?;
        Ok(domains.iter().map(|domain| domain.as_ref()).collect())
    }

    pub(super) fn get_domain_and_backup_domains(&self) -> DomainsResult<(&str, Vec<&str>)> {
        let mut domains = self.domains()?;
        match domains.pop() {
            Some(first_domain) => {
                domains.reverse();
                Ok((first_domain, domains))
            }
            None => Err(DomainsError::NoDomainsBound),
        }
    }

    /// 创建面向该存储区域的对象上传器
    pub fn uploader(&self) -> ObjectUploader {
        self.upload_manager()
            .upload_for_internal_generated_upload_token_with_regions(
                self.0.name.to_owned(),
                UploadToken::new_from_bucket(self.0.name.to_owned(), self.0.credential.to_owned(), self.config())
                    .into(),
                self.regions().ok(),
            )
    }

    /// 创建面向该存储区域的批量上传器
    pub fn batch_uploader(&self) -> BatchUploader {
        BatchUploader::new_for_bucket(self.to_owned())
    }

    /// 存储空间是否是私有的
    pub fn is_private(&self) -> HTTPResult<bool> {
        self.get_bucket_info().map(|info| info.private != 0)
    }

    fn get_bucket_info(&self) -> HTTPResult<&BucketInfo> {
        self.0.bucket_info.get_or_try_init(|| {
            let bucket_info: BucketInfo = self
                .0
                .http_client
                .get("/v2/bucketInfo", &[&self.0.http_client.config().uc_url()])
                .query("bucket".into(), self.name().into())
                .token(TokenVersion::V2, self.0.credential.borrow().into())
                .no_body()
                .send()?
                .parse_json()?;
            Ok(bucket_info)
        })
    }

    pub(super) fn rs_urls(&self) -> Vec<&str> {
        self.0
            .rs_urls
            .get_or_init(|| {
                let mut rs_urls = self
                    .region()
                    .map(|region| {
                        region
                            .rs_urls_ref(self.config().use_https())
                            .into_iter()
                            .map(|url| url.to_owned())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                rs_urls.push(self.config().rs_url().to_owned());
                rs_urls.into_boxed_slice()
            })
            .iter()
            .map(|url| url.as_str())
            .collect()
    }

    #[inline]
    pub(crate) fn upload_manager(&self) -> &UploadManager {
        &self.0.upload_manager
    }

    #[inline]
    pub(crate) fn config(&self) -> &Config {
        self.upload_manager().config()
    }

    #[inline]
    pub(crate) fn thread_pool(&self) -> Option<&Arc<ThreadPool>> {
        self.upload_manager().thread_pool()
    }

    #[inline]
    pub(crate) fn http_client(&self) -> &Client {
        &self.0.http_client
    }

    #[inline]
    pub(crate) fn credential(&self) -> &Credential {
        &self.0.credential
    }

    #[doc(hidden)]
    pub fn into_raw(self) -> *const c_void {
        Arc::into_raw(self.0).cast()
    }

    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(Arc::from_raw(ptr.cast::<BucketInner>()))
    }

    #[allow(dead_code)]
    fn ignore() {
        assert_impl!(Send: Self);
        assert_impl!(Sync: Self);
    }
}

/// 存储空间域名获取错误
#[derive(Error, Debug)]
pub enum DomainsError {
    /// 存储空间上没有绑定任何域名
    #[error("No domains bound")]
    NoDomainsBound,

    /// 获取存储空间域名错误
    #[error("Get domains error: {0}")]
    GetDomainsError(#[from] HTTPError),
}

/// 存储空间域名获取结果
pub type DomainsResult<T> = Result<T, DomainsError>;

mod domain {
    use crate::{
        credential::Credential,
        http::{Client, Result, TokenVersion},
        utils::cache_map::CacheMap,
    };
    use lazy_static::lazy_static;
    use std::{
        borrow::Borrow,
        time::{Duration, SystemTime},
    };

    lazy_static! {
        static ref QUERY_CACHE: CacheMap<QueryCacheKey, Vec<String>> = CacheMap::new(true);
    }

    #[derive(PartialEq, Hash, Clone, Debug)]
    struct QueryCacheKey(String);

    impl QueryCacheKey {
        fn new(credential: &Credential, bucket_name: &str) -> Self {
            Self(credential.access_key().to_owned() + ":" + credential.secret_key() + ":" + bucket_name)
        }
    }

    /// 该方法具有缓存机制，对同一 Access Key / Secret Key 和存储空间多次调用时，将会返回缓存结果而不会发送 HTTP 请求
    pub(super) fn query(http_client: &Client, credential: &Credential, bucket_name: &str) -> Result<Vec<String>> {
        let (domains, _) = QUERY_CACHE
            .try_get_or_insert(QueryCacheKey::new(credential, bucket_name), || {
                let results = http_client
                    .get("/v6/domain/list", &[&http_client.config().api_url()])
                    .query("tbl".into(), bucket_name.into())
                    .token(TokenVersion::V2, credential.borrow().into())
                    .no_body()
                    .send()?
                    .parse_json()?;
                Ok(Some((results, SystemTime::now() + Duration::from_secs(24 * 60 * 60))))
            })?
            .unwrap();
        Ok(domains)
    }

    #[cfg(test)]
    pub(super) fn clear_query_cache() {
        QUERY_CACHE.clear();
    }
}

impl<'a> Iterator for BucketRegionIter<'a> {
    type Item = &'a Region;

    fn next(&mut self) -> Option<Self::Item> {
        if self.itered == 0 {
            self.bucket.0.region.get().map(|region| {
                self.itered += 1;
                region.as_ref()
            })
        } else {
            self.bucket
                .0
                .backup_regions
                .get()
                .and_then(|regions| {
                    self.itered += 1;
                    regions.get(self.itered - 2)
                })
                .map(|r| r.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{region::clear_query_cache as clear_region_query_cache, region::RegionId, uploader::UploadManager},
        domain::clear_query_cache as clear_domain_query_cache,
        *,
    };
    use crate::{
        config::ConfigBuilder,
        credential::Credential,
        http::{DomainsManagerBuilder, HeadersOwned, PanickedHTTPCaller},
    };
    use qiniu_test_utils::http_call_mock::{CounterCallMock, JSONCallMock};
    use serde_json::json;
    use std::{boxed::Box, error::Error, result::Result, sync::Arc, thread};

    #[test]
    fn test_storage_bucket_set_region() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(PanickedHTTPCaller("Should not call it"))
                    .build(),
            ),
        )
        .region(Region::hua_bei())
        .build();
        assert_eq!(bucket.region()?.region_id(), Some(RegionId::Z1));
        let regions = bucket.regions()?.collect::<Vec<_>>();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions.first().unwrap().region_id(), Some(RegionId::Z1));
        Ok(())
    }

    #[test]
    fn test_storage_bucket_set_region_id() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(PanickedHTTPCaller("Should not call it"))
                    .build(),
            ),
        )
        .region_id(RegionId::Z2)
        .build();
        assert_eq!(bucket.region()?.region_id(), Some(RegionId::Z2));
        let regions = bucket.regions()?.collect::<Vec<_>>();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions.first().unwrap().region_id(), Some(RegionId::Z2));
        Ok(())
    }

    #[test]
    fn test_storage_bucket_prequery_region() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!({
                "hosts": [{
                    "io": { "src": { "main": [ "iovip.qbox.me" ] } },
                    "up": {
                        "acc": { "backup": [ "upload-jjh.qiniup.com", "upload-xs.qiniup.com" ], "main": [ "upload.qiniup.com" ] },
                        "old_acc": { "info": "compatible to non-SNI device", "main": [ "upload.qbox.me" ] },
                        "old_src": { "info": "compatible to non-SNI device", "main": [ "up.qbox.me" ] },
                        "src": { "backup": [ "up-jjh.qiniup.com", "up-xs.qiniup.com" ], "main": [ "up.qiniup.com" ] }
                    },
                    "rs": { "acc": { "main": [ "rs.qbox.me" ] } },
                    "rsf": { "acc": { "main": [ "rsf.qbox.me" ] } },
                    "api": { "acc": { "main": [ "api.qiniu.com" ] } }
                },{
                    "io": { "src": { "main": [ "iovip-z1.qbox.me" ] } },
                    "up": {
                        "acc": { "backup": [ "upload-jjh-z1.qiniup.com", "upload-xs-z1.qiniup.com" ], "main": [ "upload-z1.qiniup.com" ] },
                        "old_acc": { "info": "compatible to non-SNI device", "main": [ "upload-z1.qbox.me" ] },
                        "old_src": { "info": "compatible to non-SNI device", "main": [ "up-z1.qbox.me" ] },
                        "src": { "backup": [ "up-jjh-z1.qiniup.com", "up-xs-z1.qiniup.com" ], "main": [ "up-z1.qiniup.com" ] }
                    },
                    "rs": { "acc": { "main": [ "rs-z1.qbox.me" ] } },
                    "rsf": { "acc": { "main": [ "rsf-z1.qbox.me" ] } },
                    "api": { "acc": { "main": [ "api-z1.qiniu.com" ] } }
                }]
            }),
        ));
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(mock.clone())
                    .build(),
            ),
        )
        .auto_detect_region()?
        .build();
        assert_eq!(mock.call_called(), 1);

        let region = bucket.region()?;
        assert!(region.up_urls_ref(true).contains(&"https://up-xs.qiniup.com"));
        assert!(region
            .up_urls_owned(true)
            .contains(&Cow::Borrowed("https://up-xs.qiniup.com")));
        assert!(region.up_urls_ref(true).contains(&"https://up-jjh.qiniup.com"));
        assert!(region
            .up_urls_owned(true)
            .contains(&Cow::Borrowed("https://up-jjh.qiniup.com")));
        assert!(region.up_urls_ref(true).contains(&"https://upload.qbox.me"));
        assert!(region
            .up_urls_owned(true)
            .contains(&Cow::Borrowed("https://upload.qbox.me")));

        let regions = bucket.regions()?.collect::<Vec<_>>();
        assert_eq!(regions.len(), 2);
        assert!(regions
            .get(1)
            .unwrap()
            .up_urls_ref(true)
            .contains(&"https://up-xs-z1.qiniup.com"));
        assert!(regions
            .get(1)
            .unwrap()
            .up_urls_ref(true)
            .contains(&"https://up-jjh-z1.qiniup.com"));
        assert!(regions
            .get(1)
            .unwrap()
            .up_urls_ref(true)
            .contains(&"https://upload-z1.qbox.me"));

        assert_eq!(mock.call_called(), 1);

        Ok(())
    }

    #[test]
    fn test_storage_bucket_query_region() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!({
                "hosts": [{
                    "io": { "src": { "main": [ "iovip.qbox.me" ] } },
                    "up": {
                        "acc": { "backup": [ "upload-jjh.qiniup.com", "upload-xs.qiniup.com" ], "main": [ "upload.qiniup.com" ] },
                        "old_acc": { "info": "compatible to non-SNI device", "main": [ "upload.qbox.me" ] },
                        "old_src": { "info": "compatible to non-SNI device", "main": [ "up.qbox.me" ] },
                        "src": { "backup": [ "up-jjh.qiniup.com", "up-xs.qiniup.com" ], "main": [ "up.qiniup.com" ] }
                    },
                    "rs": { "acc": { "main": [ "rs.qbox.me" ] } },
                    "rsf": { "acc": { "main": [ "rsf.qbox.me" ] } },
                    "api": { "acc": { "main": [ "api.qiniu.com" ] } }
                },{
                    "io": { "src": { "main": [ "iovip-z2.qbox.me" ] } },
                    "up": {
                        "acc": { "backup": [ "upload-jjh-z2.qiniup.com", "upload-xs-z2.qiniup.com" ], "main": [ "upload-z2.qiniup.com" ] },
                        "old_acc": { "info": "compatible to non-SNI device", "main": [ "upload-z2.qbox.me" ] },
                        "old_src": { "info": "compatible to non-SNI device", "main": [ "up-z2.qbox.me" ] },
                        "src": { "backup": [ "up-jjh-z2.qiniup.com", "up-xs-z2.qiniup.com" ], "main": [ "up-z2.qiniup.com" ] }
                    },
                    "rs": { "acc": { "main": [ "rs-z2.qbox.me" ] } },
                    "rsf": { "acc": { "main": [ "rsf-z2.qbox.me" ] } },
                    "api": { "acc": { "main": [ "api-z2.qiniu.com" ] } }
                }]
            }),
        ));
        let bucket = Arc::new(
            BucketBuilder::new(
                "test-bucket".into(),
                get_credential(),
                UploadManager::new(
                    ConfigBuilder::default()
                        .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                        .http_request_handler(mock.clone())
                        .build(),
                ),
            )
            .build(),
        );
        assert_eq!(mock.call_called(), 0);

        let mut threads = Vec::with_capacity(4);
        {
            let bucket = bucket.clone();
            threads.push(thread::spawn(move || {
                assert!(bucket
                    .region()
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://up-xs.qiniup.com"));
            }));
        }

        {
            let bucket = bucket.clone();
            threads.push(thread::spawn(move || {
                assert!(bucket
                    .region()
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://up-jjh.qiniup.com"));
            }));
        }

        {
            let bucket = bucket.clone();
            threads.push(thread::spawn(move || {
                assert!(bucket
                    .region()
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://upload.qbox.me"));
            }));
        }

        {
            threads.push(thread::spawn(move || {
                let regions = bucket.regions().unwrap().collect::<Vec<_>>();
                assert_eq!(regions.len(), 2);
                assert!(regions
                    .get(1)
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://up-xs-z2.qiniup.com"));
                assert!(regions
                    .get(1)
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://up-jjh-z2.qiniup.com"));
                assert!(regions
                    .get(1)
                    .unwrap()
                    .up_urls_ref(true)
                    .contains(&"https://upload-z2.qbox.me"));
            }));
        }

        threads.into_iter().for_each(|thread| thread.join().unwrap());
        assert_eq!(mock.call_called(), 1);

        Ok(())
    }

    #[test]
    fn test_storage_bucket_set_domain() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(PanickedHTTPCaller("Should not call it"))
                    .build(),
            ),
        )
        .prepend_domain("abc.com")?
        .prepend_domain("def.com")?
        .build();
        assert_eq!(bucket.domains()?.len(), 2);
        assert_eq!(bucket.domains()?.get(0), Some(&"def.com"));
        assert_eq!(bucket.domains()?.get(1), Some(&"abc.com"));
        Ok(())
    }

    #[test]
    fn test_storage_bucket_prequery_domain() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!(["abc.com", "def.com"]),
        ));
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(mock.clone())
                    .build(),
            ),
        )
        .auto_detect_domains()?
        .build();
        assert_eq!(mock.call_called(), 1);
        assert!(bucket.domains()?.contains(&"abc.com"));
        assert!(bucket.domains()?.contains(&"def.com"));
        assert_eq!(mock.call_called(), 1);
        Ok(())
    }

    #[test]
    fn test_storage_cloned_bucket_query_domain() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!(["abc.com", "def.com"]),
        ));
        let bucket = Arc::new(
            BucketBuilder::new(
                "test-bucket".into(),
                get_credential(),
                UploadManager::new(
                    ConfigBuilder::default()
                        .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                        .http_request_handler(mock.clone())
                        .build(),
                ),
            )
            .build(),
        );
        assert_eq!(mock.call_called(), 0);

        let mut threads = Vec::with_capacity(3);
        {
            let bucket = bucket.clone();
            threads.push(thread::spawn(move || {
                assert!(bucket.domains().unwrap().contains(&"abc.com"));
            }));
        }

        {
            threads.push(thread::spawn(move || {
                assert!(bucket.domains().unwrap().contains(&"def.com"));
            }));
        }

        threads.into_iter().for_each(|thread| thread.join().unwrap());
        assert_eq!(mock.call_called(), 1);
        Ok(())
    }

    #[test]
    fn test_storage_independent_bucket_query_domain() -> Result<(), Box<dyn Error>> {
        clear_query_cache();

        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!(["abc.com", "def.com"]),
        ));
        let bucket1 = Arc::new(
            BucketBuilder::new(
                "test-bucket".into(),
                get_credential(),
                UploadManager::new(
                    ConfigBuilder::default()
                        .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                        .http_request_handler(mock.clone())
                        .build(),
                ),
            )
            .build(),
        );
        let bucket2 = Arc::new(
            BucketBuilder::new(
                "test-bucket".into(),
                get_credential(),
                UploadManager::new(
                    ConfigBuilder::default()
                        .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                        .http_request_handler(mock.clone())
                        .build(),
                ),
            )
            .build(),
        );
        assert_eq!(mock.call_called(), 0);

        let mut threads = Vec::with_capacity(3);
        {
            threads.push(thread::spawn(move || {
                assert!(bucket1.domains().unwrap().contains(&"abc.com"));
            }));
        }

        {
            threads.push(thread::spawn(move || {
                assert!(bucket2.domains().unwrap().contains(&"def.com"));
            }));
        }

        threads.into_iter().for_each(|thread| thread.join().unwrap());
        assert_eq!(mock.call_called(), 1);
        Ok(())
    }

    #[test]
    fn test_storage_bucket_is_private() -> Result<(), Box<dyn Error>> {
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(JSONCallMock::new(200, HeadersOwned::new(), json!({"private": 1})))
                    .build(),
            ),
        )
        .build();
        assert!(bucket.is_private()?);
        Ok(())
    }

    fn get_credential() -> Credential {
        Credential::new("abcdefghklmnopq", "1234567890")
    }

    fn clear_query_cache() {
        clear_domain_query_cache();
        clear_region_query_cache();
    }
}
