//! 对象模块

use super::{
    bucket::Bucket,
    resource::{Copy, Delete, Move, Stat, ToURI},
    uploader::{ObjectUploader, UploadPolicyBuilder, UploadToken},
};
use crate::{
    http::{Result as HTTPResult, TokenVersion},
    utils::base64,
};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{
    borrow::Cow,
    fmt,
    time::{Duration, SystemTime},
};

/// 对象
///
/// 用于表示存储空间中的一个对象，可用来获取对象信息或对对象进行操作。
pub struct Object {
    key: Cow<'static, str>,
    bucket: Bucket,
    encoded_entry_uri: OnceCell<String>,
}

impl Object {
    pub(super) fn new(bucket: Bucket, key: Cow<'static, str>) -> Self {
        Self {
            bucket,
            key,
            encoded_entry_uri: OnceCell::new(),
        }
    }

    /// 获取对象所在存储空间信息
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }

    /// 获取对象名称
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 获取对象详细信息
    pub fn get_info(&self) -> HTTPResult<ObjectInfo> {
        self.bucket
            .http_client()
            .get(&Stat::new(self).to_uri(), &self.bucket.rs_urls())
            .idempotent()
            .token(TokenVersion::V2, self.bucket.credential().into())
            .accept_json()
            .no_body()
            .send()?
            .parse_json()
    }

    /// 删除对象
    pub fn delete(&self) -> HTTPResult<()> {
        self.bucket
            .http_client()
            .post(&Delete::new(self).to_uri(), &self.bucket.rs_urls())
            .token(TokenVersion::V2, self.bucket.credential().into())
            .no_body()
            .send()?
            .ignore_body();
        Ok(())
    }

    /// 移动当前对象到指定的目标对象
    pub fn move_to(&self, target_object: &Object, force: bool) -> HTTPResult<()> {
        self.bucket
            .http_client()
            .post(&Move::new(self, target_object, force).to_uri(), &self.bucket.rs_urls())
            .token(TokenVersion::V2, self.bucket.credential().into())
            .no_body()
            .send()?
            .ignore_body();
        Ok(())
    }

    /// 复制当前对象到指定的目标对象
    pub fn copy_to(&self, target_object: &Object, force: bool) -> HTTPResult<()> {
        let op = Copy::new(self, target_object, force).to_uri();
        let rs_urls = self.bucket.rs_urls();
        let mut request_builder = self.bucket.http_client().post(&op, &rs_urls);
        if force {
            request_builder = request_builder.idempotent();
        }
        request_builder
            .token(TokenVersion::V2, self.bucket.credential().into())
            .no_body()
            .send()?
            .ignore_body();
        Ok(())
    }

    pub(super) fn encoded_entry_uri(&self) -> &str {
        self.encoded_entry_uri.get_or_init(|| {
            let entry_uri = self.bucket.name().to_owned() + ":" + self.key.as_ref();
            base64::urlsafe(entry_uri.as_bytes())
        })
    }

    /// 创建面向该对象的对象上传器
    pub fn uploader(&self) -> ObjectUploader {
        self.bucket
            .upload_manager()
            .upload_for_internal_generated_upload_token_with_regions(
                self.bucket.name().into(),
                UploadToken::new(
                    UploadPolicyBuilder::new_policy_for_object(self.bucket.name(), &self.key, self.bucket.config())
                        .save_as(self.key.to_owned().into_owned(), true)
                        .build(),
                    self.bucket.credential().to_owned(),
                )
                .into(),
                self.bucket.regions().ok(),
            )
            .key(Cow::Borrowed(self.key.as_ref()))
    }
}

/// 对象详细信息
#[derive(Deserialize)]
pub struct ObjectInfo {
    fsize: u64,

    hash: String,

    #[serde(rename(deserialize = "mimeType"))]
    mime_type: String,

    #[serde(rename(deserialize = "putTime"))]
    put_time: u64,
}

impl ObjectInfo {
    /// 获取对象尺寸
    ///
    /// 单位为字节
    #[inline]
    pub fn size(&self) -> u64 {
        self.fsize
    }

    /// 获取对象 HASH 值
    ///
    /// 一般返回该对象内容的 Etag 值
    #[inline]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// 获取对象的 MIME 类型
    #[inline]
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// 获取对象的创建时间
    #[inline]
    pub fn uploaded_at(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_nanos(self.put_time * 100)
    }

    /// 获取对象的创建时间
    ///
    /// 与 `uploaded_at()` 返回相同的内容
    #[inline]
    pub fn put_time(&self) -> SystemTime {
        self.uploaded_at()
    }
}

impl fmt::Debug for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectInfo")
            .field("size", &self.size())
            .field("hash", &self.hash())
            .field("mime_type", &self.mime_type())
            .field("put_time", &self.put_time())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{bucket::BucketBuilder, uploader::UploadManager};
    use crate::{
        config::ConfigBuilder,
        credential::Credential,
        http::{DomainsManagerBuilder, ErrorKind as HTTPErrorKind, Headers},
    };
    use chrono::{offset::Utc, DateTime};
    use qiniu_test_utils::http_call_mock::JSONCallMock;
    use serde_json::json;
    use std::{boxed::Box, error::Error, result::Result};

    #[test]
    fn test_storage_object_stat() -> Result<(), Box<dyn Error>> {
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(JSONCallMock::new(
                        200,
                        Headers::new(),
                        json!({
                            "fsize":        5_122_935u64,
                            "hash":         "ljfockr0lOil_bZfyaI2ZY78HWoH",
                            "mimeType":     "application/octet-stream",
                            "putTime":      13_603_956_734_587_420u64,
                            "md5":          "e41714a18899cf59c200a9bddfa78b95"
                        }),
                    ))
                    .build(),
            ),
        )
        .build();
        let object_info = bucket.object("test-object").get_info()?;
        assert_eq!(object_info.size(), 5_122_935);
        assert_eq!(object_info.hash(), "ljfockr0lOil_bZfyaI2ZY78HWoH");
        assert_eq!(object_info.mime_type(), "application/octet-stream");
        assert_eq!(
            DateTime::<Utc>::from(object_info.uploaded_at()).to_rfc3339(),
            "2013-02-09T07:41:13.458742+00:00"
        );
        Ok(())
    }

    #[test]
    fn test_storage_object_delete() -> Result<(), Box<dyn Error>> {
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(JSONCallMock::new(200, Headers::new(), json!({})))
                    .build(),
            ),
        )
        .build();
        bucket.object("test-object").delete()?;
        Ok(())
    }

    #[test]
    fn test_storage_object_delete_with_612_response() -> Result<(), Box<dyn Error>> {
        let bucket = BucketBuilder::new(
            "test-bucket".into(),
            get_credential(),
            UploadManager::new(
                ConfigBuilder::default()
                    .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
                    .http_request_handler(JSONCallMock::new(
                        612,
                        Headers::new(),
                        json!({"error": "Document not found"}),
                    ))
                    .build(),
            ),
        )
        .build();
        let err = bucket.object("test-object").delete().unwrap_err();
        if let HTTPErrorKind::ResponseStatusCodeError(612, message) = err.error_kind() {
            assert_eq!(message.as_ref(), "Document not found");
            return Ok(());
        }
        panic!("Should not reach here");
    }

    fn get_credential() -> Credential {
        Credential::new("abcdefghklmnopq", "1234567890")
    }
}
