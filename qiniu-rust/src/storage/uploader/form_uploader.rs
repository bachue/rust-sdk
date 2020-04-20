use super::{
    upload_response_callback, TokenizedUploadLogger, UpType, UploadError, UploadLoggerRecordBuilder, UploadManager,
    UploadResponse, UploadToken,
};
use crate::{
    http::{Error as HTTPError, Result as HTTPResult, RetryKind},
    utils::crc32,
};
use mime::Mime;
use qiniu_multipart::client::lazy::Multipart;
use serde_json::Value;
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{Read, Seek, SeekFrom},
    result::Result,
};

pub(super) struct FormUploaderBuilder<'u> {
    upload_manager: &'u UploadManager,
    up_urls_list: &'u [Box<[Box<str>]>],
    multipart: Multipart<'u, 'u>,
    on_uploading_progress: Option<&'u dyn Fn(u64, Option<u64>)>,
    upload_logger: Option<TokenizedUploadLogger>,
}

#[must_use]
pub(super) struct FormUploader<'u> {
    upload_manager: &'u UploadManager,
    up_urls_list: &'u [Box<[Box<str>]>],
    content_type: String,
    body: Vec<u8>,
    on_uploading_progress: Option<&'u dyn Fn(u64, Option<u64>)>,
    upload_logger: Option<TokenizedUploadLogger>,
}

impl<'u> FormUploaderBuilder<'u> {
    pub(super) fn new(
        upload_manager: &'u UploadManager,
        upload_token: &'u UploadToken,
        up_urls_list: &'u [Box<[Box<str>]>],
    ) -> FormUploaderBuilder<'u> {
        let upload_token = upload_token.to_string();
        let mut uploader = FormUploaderBuilder {
            upload_manager,
            up_urls_list,
            multipart: Multipart::new(),
            on_uploading_progress: None,
            upload_logger: upload_manager.config().upload_logger().as_ref().map(|upload_logger| {
                upload_logger.tokenize(upload_token.to_owned().into(), upload_manager.http_client().to_owned())
            }),
        };
        uploader.multipart.add_text("token", upload_token);
        uploader
    }

    pub(super) fn key(mut self, key: Cow<'u, str>) -> FormUploaderBuilder<'u> {
        self.multipart.add_text("key", key);
        self
    }

    pub(super) fn var(mut self, var_key: &str, var_value: Cow<'u, str>) -> FormUploaderBuilder<'u> {
        self.multipart.add_text("x:".to_owned() + var_key, var_value);
        self
    }

    pub(super) fn metadata(mut self, metadata_key: &str, metadata_value: Cow<'u, str>) -> FormUploaderBuilder<'u> {
        self.multipart
            .add_text("x-qn-meta-".to_owned() + metadata_key, metadata_value);
        self
    }

    pub(super) fn on_uploading_progress(mut self, callback: &'u dyn Fn(u64, Option<u64>)) -> FormUploaderBuilder<'u> {
        self.on_uploading_progress = Some(callback);
        self
    }

    pub(super) fn seekable_stream(
        mut self,
        mut stream: impl Read + Seek + 'u,
        file_name: Cow<'u, str>,
        mime: Option<Mime>,
        checksum_enabled: bool,
    ) -> Result<FormUploader<'u>, UploadError> {
        let mut crc32: Option<u32> = None;
        if checksum_enabled {
            crc32 = Some(crc32::from(&mut stream)?);
            stream.seek(SeekFrom::Start(0))?;
        }
        let file_name = if file_name.is_empty() { None } else { Some(file_name) };
        self.multipart.add_stream("file", stream, file_name, mime);
        if let Some(crc32) = crc32 {
            self.multipart.add_text("crc32", crc32.to_string());
        }
        self.upload_multipart()
    }

    pub(super) fn stream(
        mut self,
        stream: impl Read + 'u,
        file_name: Cow<'u, str>,
        mime: Option<Mime>,
        crc32: Option<u32>,
    ) -> Result<FormUploader<'u>, UploadError> {
        let file_name = if file_name.is_empty() { None } else { Some(file_name) };
        self.multipart.add_stream("file", stream, file_name, mime);
        if let Some(crc32) = crc32 {
            self.multipart.add_text("crc32", crc32.to_string());
        }
        self.upload_multipart()
    }

    fn upload_multipart(mut self) -> Result<FormUploader<'u>, UploadError> {
        let mut fields = self.multipart.prepare().map_err(|err| err.error)?;
        let mut body = Vec::with_capacity(
            self.upload_manager
                .config()
                .upload_threshold()
                .try_into()
                .unwrap_or(1 << 22),
        );
        fields.read_to_end(&mut body)?;
        Ok(FormUploader {
            upload_manager: self.upload_manager,
            up_urls_list: self.up_urls_list,
            content_type: "multipart/form-data; boundary=".to_owned() + fields.boundary(),
            body,
            on_uploading_progress: self.on_uploading_progress,
            upload_logger: self.upload_logger,
        })
    }
}

impl<'u> FormUploader<'u> {
    pub(super) fn send(&self) -> HTTPResult<UploadResponse> {
        let mut prev_err: Option<HTTPError> = None;
        for up_urls in self.up_urls_list.iter() {
            match self.send_form_request(&up_urls.iter().map(|url| url.as_ref()).collect::<Box<[&str]>>()) {
                Ok(value) => {
                    return Ok(value);
                }
                Err(err) => match err.retry_kind() {
                    RetryKind::RetryableError | RetryKind::HostUnretryableError | RetryKind::ZoneUnretryableError => {
                        prev_err = Some(err);
                    }
                    _ => {
                        return Err(err);
                    }
                },
            }
        }

        Err(prev_err.expect("FormUploader::send() should try at lease once, but not"))
    }

    fn send_form_request(&self, up_urls: &[&str]) -> HTTPResult<UploadResponse> {
        let upload_result = self
            .upload_manager
            .http_client()
            .post("/", up_urls)
            .idempotent()
            .on_uploading_progress(&|uploaded, total| {
                if let Some(on_uploading_progress) = &self.on_uploading_progress {
                    (on_uploading_progress)(uploaded, Some(total));
                }
            })
            .on_response(&|response, duration| {
                let result = upload_response_callback(response);
                if result.is_ok() {
                    if let Some(upload_logger) = &self.upload_logger {
                        let _ = upload_logger.log(
                            UploadLoggerRecordBuilder::default()
                                .response(response)
                                .duration(duration)
                                .up_type(UpType::Form)
                                .sent(self.body.len().try_into().unwrap_or(u64::max_value()))
                                .total_size(self.body.len().try_into().unwrap_or(u64::max_value()))
                                .build(),
                        );
                    }
                }
                result
            })
            .on_error(&|base_url, err, duration| {
                if let Some(upload_logger) = &self.upload_logger {
                    let _ = upload_logger.log({
                        let mut builder = UploadLoggerRecordBuilder::default()
                            .duration(duration)
                            .up_type(UpType::Form)
                            .http_error(err)
                            .total_size(self.body.len().try_into().unwrap_or(u64::max_value()));
                        if let Some(base_url) = base_url {
                            builder = builder.host(base_url);
                        }
                        builder.build()
                    });
                }
            })
            .accept_json()
            .raw_body(Cow::Borrowed(&self.content_type), Cow::Borrowed(&self.body))
            .send()?
            .try_parse_json::<Value>()?;
        match upload_result {
            Ok(value) => Ok(value.into()),
            Err(bytes) => Ok(bytes.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{UploadManager, UploadPolicyBuilder, UploadToken},
        *,
    };
    use crate::{
        config::ConfigBuilder,
        credential::Credential,
        http::{DomainsManagerBuilder, HeadersOwned},
    };
    use qiniu_test_utils::{
        http_call_mock::{CounterCallMock, ErrorResponseMock, JSONCallMock},
        temp_file::create_temp_file,
    };
    use serde_json::json;
    use std::{boxed::Box, error::Error, result::Result};

    #[test]
    fn test_storage_uploader_form_uploader_upload_seekable_stream() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(JSONCallMock::new(
            200,
            HeadersOwned::new(),
            json!({"key": "abc", "hash": "def"}),
        ));
        let config = ConfigBuilder::default()
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        let result = FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .seekable_stream(create_temp_file(1 << 10)?, "".into(), None, true)?
        .send()?;
        assert_eq!(result.key(), Some("abc"));
        assert_eq!(result.hash(), Some("def"));
        assert_eq!(mock.call_called(), 1);
        Ok(())
    }

    #[test]
    fn test_storage_uploader_form_uploader_upload_seekable_stream_with_500_error() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(ErrorResponseMock::new(500, "test error"));
        let config = ConfigBuilder::default()
            .http_request_retries(3)
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        assert!(FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .seekable_stream(create_temp_file(1 << 10)?, "".into(), None, true)?
        .send()
        .is_err());
        assert_eq!(mock.call_called(), 16);
        Ok(())
    }

    #[test]
    fn test_storage_uploader_form_uploader_upload_seekable_stream_with_503_error() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(ErrorResponseMock::new(503, "test error"));
        let config = ConfigBuilder::default()
            .http_request_retries(3)
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        assert!(FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .seekable_stream(create_temp_file(1 << 10)?, "".into(), None, true)?
        .send()
        .is_err());
        assert_eq!(mock.call_called(), 4);
        Ok(())
    }

    #[test]
    fn test_storage_uploader_form_uploader_upload_stream_with_500_error() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(ErrorResponseMock::new(500, "test error"));
        let config = ConfigBuilder::default()
            .http_request_retries(3)
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        assert!(FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .stream(create_temp_file(1 << 10)?, "".into(), None, None)?
        .send()
        .is_err());
        assert_eq!(mock.call_called(), 16);
        Ok(())
    }

    #[test]
    fn test_storage_uploader_form_uploader_upload_stream_with_503_error() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(ErrorResponseMock::new(503, "test error"));
        let config = ConfigBuilder::default()
            .http_request_retries(3)
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        assert!(FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .stream(create_temp_file(1 << 10)?, "".into(), None, None)?
        .send()
        .is_err());
        assert_eq!(mock.call_called(), 4);
        Ok(())
    }

    #[test]
    fn test_storage_uploader_form_uploader_upload_stream_with_400_incorrect_zone_error() -> Result<(), Box<dyn Error>> {
        let mock = CounterCallMock::new(ErrorResponseMock::new(400, "incorrect region, please use z3h1.com"));
        let config = ConfigBuilder::default()
            .http_request_retries(3)
            .http_request_handler(mock.clone())
            .upload_logger(None)
            .domains_manager(DomainsManagerBuilder::default().disable_url_resolution().build())
            .build();
        let policy = UploadPolicyBuilder::new_policy_for_bucket("test-bucket", &config).build();
        assert!(FormUploaderBuilder::new(
            &UploadManager::new(config),
            &UploadToken::new(policy, get_credential()),
            &[
                vec![Box::from("http://z1h1.com"), Box::from("http://z1h2.com")].into(),
                vec![Box::from("http://z2h1.com"), Box::from("http://z2h2.com")].into(),
            ],
        )
        .key("test:file".into())
        .stream(create_temp_file(1 << 10)?, "".into(), None, None)?
        .send()
        .is_err());
        assert_eq!(mock.call_called(), 2);
        Ok(())
    }

    fn get_credential() -> Credential {
        Credential::new("abcdefghklmnopq", "1234567890")
    }
}
