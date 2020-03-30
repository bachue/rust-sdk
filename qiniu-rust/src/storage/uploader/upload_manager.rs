//! 上传管理器
//!
//! 封装上传相关功能

use super::{
    super::{
        bucket::Bucket,
        region::Region,
        uploader::{UploadPolicy, UploadToken, UploadTokenParseError},
    },
    BatchUploader, BucketUploaderBuilder, FileUploader,
};
use crate::{config::Config, credential::Credential, utils::ron::Ron};
use assert_impl::assert_impl;
use std::{borrow::Cow, result::Result};
use thiserror::Error;

/// 上传管理器
///
/// 上传管理器更接近于一个上传入口，帮助构建存储空间上传器或文件上传器，而本身并不具有实质管理功能
#[derive(Clone)]
pub struct UploadManager {
    config: Config,
}

impl UploadManager {
    /// 创建新的上传管理器
    pub fn new(config: Config) -> Self {
        UploadManager { config }
    }

    /// 创建存储空间上传器生成器
    pub fn for_bucket(&self, bucket: &Bucket) -> BucketUploaderBuilder {
        BucketUploaderBuilder::new(
            bucket.name().into(),
            bucket
                .regions()
                .map(|iter| Self::extract_up_urls_list_from_regions(iter, self.config.use_https()))
                .unwrap_or_else(|_| Self::all_possible_up_urls_list(self.config.use_https())),
            self.config.to_owned(),
        )
    }

    /// 根据存储空间名称和对应的 Access Key 创建存储空间上传器生成器
    pub fn for_bucket_name<'b>(
        &self,
        bucket_name: impl Into<Cow<'b, str>>,
        access_key: impl AsRef<str>,
    ) -> BucketUploaderBuilder {
        let bucket_name = bucket_name.into();
        let up_urls_list = Region::query(bucket_name.as_ref(), access_key.as_ref(), self.config.to_owned())
            .map(|regions| Self::extract_up_urls_list_from_regions(regions.iter(), self.config.use_https()))
            .unwrap_or_else(|_| Self::all_possible_up_urls_list(self.config.use_https()));
        BucketUploaderBuilder::new(bucket_name.into_owned().into(), up_urls_list, self.config.to_owned())
    }

    fn extract_up_urls_list_from_regions<'a>(
        iter: impl Iterator<Item = &'a Region>,
        use_https: bool,
    ) -> Box<[Box<[Box<str>]>]> {
        iter.map(|region| {
            region
                .up_urls_owned(use_https)
                .into_iter()
                .map(|url| url.into_owned().into_boxed_str())
                .collect::<Box<[_]>>()
        })
        .collect()
    }

    fn all_possible_up_urls_list(use_https: bool) -> Box<[Box<[Box<str>]>]> {
        Region::all()
            .iter()
            .map(|region| {
                region
                    .up_urls_owned(use_https)
                    .into_iter()
                    .map(|url| url.into_owned().into_boxed_str())
                    .collect::<Box<[_]>>()
            })
            .collect()
    }

    /// 根据上传凭证创建文件上传器
    pub fn upload_for_upload_token<'u>(
        &self,
        upload_token: impl Into<UploadToken<'u>>,
    ) -> CreateUploaderResult<FileUploader<'u>> {
        let upload_token = upload_token.into();
        let access_key = upload_token.access_key()?;
        let policy = upload_token.policy()?;
        if let Some(bucket_name) = policy.bucket() {
            Ok(FileUploader::new(
                Ron::Owned(self.for_bucket_name(bucket_name.to_owned(), access_key).build()),
                upload_token.to_string().into(),
            ))
        } else {
            Err(CreateUploaderError::BucketIsMissingInUploadToken)
        }
    }

    /// 根据上传策略和认证信息创建文件上传器
    pub fn upload_for_upload_policy<'u>(
        &self,
        upload_policy: UploadPolicy<'u>,
        credential: impl Into<Cow<'u, Credential>>,
    ) -> CreateUploaderResult<FileUploader<'u>> {
        self.upload_for_upload_token(UploadToken::new(upload_policy, credential.into()))
    }

    /// 根据存储空间和认证信息创建文件上传器
    pub fn upload_for_bucket<'u>(
        &self,
        bucket: impl Into<Cow<'u, str>>,
        credential: impl Into<Cow<'u, Credential>>,
    ) -> FileUploader<'u> {
        self.upload_for_upload_token(UploadToken::new_from_bucket(
            bucket.into(),
            credential.into(),
            &self.config,
        ))
        .unwrap()
    }

    /// 根据上传凭证创建批量文件上传器
    pub fn batch_uploader_for_upload_token<'u>(
        &self,
        upload_token: impl Into<UploadToken<'u>>,
    ) -> CreateUploaderResult<BatchUploader> {
        let upload_token = upload_token.into();
        let access_key = upload_token.access_key()?;
        let policy = upload_token.policy()?;
        if let Some(bucket_name) = policy.bucket() {
            Ok(BatchUploader::new(
                &self.for_bucket_name(bucket_name.to_owned(), access_key).build(),
                upload_token.to_string(),
            ))
        } else {
            Err(CreateUploaderError::BucketIsMissingInUploadToken)
        }
    }

    /// 根据上传策略和认证信息创建批量文件上传器
    pub fn batch_uploader_for_upload_policy<'u>(
        &self,
        upload_policy: UploadPolicy<'u>,
        credential: impl Into<Cow<'u, Credential>>,
    ) -> CreateUploaderResult<BatchUploader> {
        self.batch_uploader_for_upload_token(UploadToken::new(upload_policy, credential))
    }

    /// 根据存储空间和认证信息创建批量文件上传器
    pub fn batch_uploader_for_bucket<'u>(
        &self,
        bucket: impl Into<Cow<'u, str>>,
        credential: impl Into<Cow<'u, Credential>>,
    ) -> BatchUploader {
        self.batch_uploader_for_upload_token(UploadToken::new_from_bucket(bucket, credential, &self.config))
            .unwrap()
    }

    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    fn ignore() {
        assert_impl!(Send: Self);
        assert_impl!(Sync: Self);
    }
}

/// 创建上传器错误
#[derive(Error, Debug)]
pub enum CreateUploaderError {
    /// 上传凭证解析错误
    #[error("Failed to parse upload token: {0}")]
    UploadTokenParseError(#[from] UploadTokenParseError),
    /// 七牛 API 调用错误
    #[error("Qiniu API call error: {0}")]
    QiniuAPIError(#[from] crate::http::Error),
    /// 上传凭证中不包含存储空间信息
    #[error("Bucket is missing in upload token")]
    BucketIsMissingInUploadToken,
}

/// 创建上传器结果
pub type CreateUploaderResult<T> = Result<T, CreateUploaderError>;
