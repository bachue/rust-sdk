use crate::utils::qiniu_ng_str_t;
use libc::c_void;
use qiniu_ng::storage::uploader::UploadResponse;
use std::{mem::transmute, ptr::null_mut};
use tap::TapOps;

/// @brief 上传响应
/// @details
///     上传响应实例对上传响应中的响应体进行封装，提供一些辅助方法。
///     当 `qiniu_ng_upload_response_t` 使用完毕后，请务必调用 `qiniu_ng_upload_response_free()` 方法释放内存
/// @note 该结构体内部状态不可变，因此可以跨线程使用
#[repr(C)]
#[derive(Copy, Clone)]
pub struct qiniu_ng_upload_response_t(*mut c_void);

impl Default for qiniu_ng_upload_response_t {
    #[inline]
    fn default() -> Self {
        Self(null_mut())
    }
}

impl qiniu_ng_upload_response_t {
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

impl From<qiniu_ng_upload_response_t> for Option<Box<UploadResponse>> {
    fn from(upload_response: qiniu_ng_upload_response_t) -> Self {
        if upload_response.is_null() {
            None
        } else {
            Some(unsafe { Box::from_raw(transmute(upload_response)) })
        }
    }
}

impl From<Option<Box<UploadResponse>>> for qiniu_ng_upload_response_t {
    fn from(upload_response: Option<Box<UploadResponse>>) -> Self {
        upload_response
            .map(|upload_response| upload_response.into())
            .unwrap_or_default()
    }
}

impl From<Box<UploadResponse>> for qiniu_ng_upload_response_t {
    fn from(upload_response: Box<UploadResponse>) -> Self {
        unsafe { transmute(Box::into_raw(upload_response)) }
    }
}

/// @brief 获取上传响应中的对象名称
/// @param[in] upload_response 上传响应实例
/// @retval qiniu_ng_str_t 对象名称
/// @note 这里返回的 `qiniu_ng_str_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_is_null()` 进行判断
/// @warning 当 `qiniu_ng_str_t` 使用完毕后，请务必调用 `qiniu_ng_str_free()` 方法释放内存
#[no_mangle]
pub extern "C" fn qiniu_ng_upload_response_get_key(upload_response: qiniu_ng_upload_response_t) -> qiniu_ng_str_t {
    let upload_response = Option::<Box<UploadResponse>>::from(upload_response).unwrap();
    unsafe { qiniu_ng_str_t::from_optional_str_unchecked(upload_response.key()) }.tap(|_| {
        let _ = qiniu_ng_upload_response_t::from(upload_response);
    })
}

/// @brief 获取上传响应中的校验和
/// @param[in] upload_response 上传响应实例
/// @retval qiniu_ng_str_t 校验和，该校验和一般是 Etag
/// @note 这里返回的 `qiniu_ng_str_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_is_null()` 进行判断
/// @warning 当 `qiniu_ng_str_t` 使用完毕后，请务必调用 `qiniu_ng_str_free()` 方法释放内存
#[no_mangle]
pub extern "C" fn qiniu_ng_upload_response_get_hash(upload_response: qiniu_ng_upload_response_t) -> qiniu_ng_str_t {
    let upload_response = Option::<Box<UploadResponse>>::from(upload_response).unwrap();
    unsafe { qiniu_ng_str_t::from_optional_str_unchecked(upload_response.hash()) }.tap(|_| {
        let _ = qiniu_ng_upload_response_t::from(upload_response);
    })
}

/// @brief 获取上传响应的字符串
/// @param[in] upload_response 上传响应实例
/// @retval qiniu_ng_str_t 上传响应字符串，一般是 JSON 格式的
/// @warning 当 `qiniu_ng_str_t` 使用完毕后，请务必调用 `qiniu_ng_str_free()` 方法释放内存
#[no_mangle]
pub extern "C" fn qiniu_ng_upload_response_get_string(upload_response: qiniu_ng_upload_response_t) -> qiniu_ng_str_t {
    let upload_response = Option::<Box<UploadResponse>>::from(upload_response).unwrap();
    unsafe { qiniu_ng_str_t::from_string_unchecked(upload_response.to_string()) }.tap(|_| {
        let _ = qiniu_ng_upload_response_t::from(upload_response);
    })
}

/// @brief 释放上传响应实例
/// @param[in,out] upload_response 上传响应实例地址，释放完毕后该实例将不再可用
#[no_mangle]
pub extern "C" fn qiniu_ng_upload_response_free(upload_response: *mut qiniu_ng_upload_response_t) {
    if let Some(upload_response) = unsafe { upload_response.as_mut() } {
        let _ = Option::<Box<UploadResponse>>::from(*upload_response);
        *upload_response = qiniu_ng_upload_response_t::default();
    }
}

/// @brief 判断上传响应实例是否已经被释放
/// @param[in] upload_response 上传响应实例
/// @retval bool 如果返回 `true` 则表示上传响应实例已经被释放，该实例不再可用
#[no_mangle]
pub extern "C" fn qiniu_ng_upload_response_is_freed(upload_response: qiniu_ng_upload_response_t) -> bool {
    upload_response.is_null()
}
