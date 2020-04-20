use crate::{
    string::UCString,
    utils::{qiniu_ng_str_map_t, qiniu_ng_str_t, QiniuNgStrMap},
};
use libc::c_void;
use qiniu_ng::storage::url::HeaderInfo;
use std::{collections::HashMap, mem::transmute, ptr::null_mut};
use tap::TapOps;

/// @brief Header 信息
/// @details 用于封装访问下载 URL 时获得的 Header 信息
/// @note
///   * 调用 `qiniu_ng_object_head()` 函数创建 `qiniu_ng_header_info_t` 实例。
///   * 当 `qiniu_ng_header_info_t` 使用完毕后，请务必调用 `qiniu_ng_header_info_free()` 方法释放内存。
/// @note
///   该结构体可以跨线程使用，SDK 确保其使用的线程安全
#[repr(C)]
#[derive(Copy, Clone)]
pub struct qiniu_ng_header_info_t(*mut c_void);

impl Default for qiniu_ng_header_info_t {
    #[inline]
    fn default() -> Self {
        Self(null_mut())
    }
}

impl qiniu_ng_header_info_t {
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

impl From<qiniu_ng_header_info_t> for Option<Box<HeaderInfo>> {
    fn from(header_info: qiniu_ng_header_info_t) -> Self {
        if header_info.is_null() {
            None
        } else {
            Some(unsafe { Box::from_raw(transmute(header_info)) })
        }
    }
}

impl From<Option<Box<HeaderInfo>>> for qiniu_ng_header_info_t {
    fn from(header_info: Option<Box<HeaderInfo>>) -> Self {
        header_info.map(|header_info| header_info.into()).unwrap_or_default()
    }
}

impl From<Box<HeaderInfo>> for qiniu_ng_header_info_t {
    fn from(header_info: Box<HeaderInfo>) -> Self {
        unsafe { transmute(Box::into_raw(header_info)) }
    }
}

/// @brief 释放 Header 信息
/// @param[in,out] header_info Header 信息实例地址，释放完毕后该信息实例将不再可用
#[no_mangle]
pub extern "C" fn qiniu_ng_header_info_free(header_info: *mut qiniu_ng_header_info_t) {
    if let Some(header_info) = unsafe { header_info.as_mut() } {
        let _ = Option::<Box<HeaderInfo>>::from(*header_info);
        *header_info = qiniu_ng_header_info_t::default();
    }
}

/// @brief 获取 Header 信息的 Content-Type 字段
/// @param[in] header_info Header 信息
/// @retval qiniu_ng_str_t 返回 Header 信息的 Content-Type 字段
/// @note 这里返回的 `qiniu_ng_str_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_is_null()` 进行判断
/// @warning 务必记得 `qiniu_ng_str_t` 需要在使用完毕后调用 `qiniu_ng_str_free()` 释放内存。
#[no_mangle]
pub extern "C" fn qiniu_ng_header_info_get_content_type(header_info: qiniu_ng_header_info_t) -> qiniu_ng_str_t {
    let header_info = Option::<Box<HeaderInfo>>::from(header_info).unwrap();
    unsafe { qiniu_ng_str_t::from_optional_str_unchecked(header_info.content_type()) }.tap(|_| {
        let _ = qiniu_ng_header_info_t::from(header_info);
    })
}

/// @brief 获取 Header 信息的 Content-Length 字段
/// @param[in] header_info Header 信息
/// @retval qiniu_ng_str_t 返回 Header 信息的 Content-Length 字段
/// @note 这里返回的 `qiniu_ng_str_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_is_null()` 进行判断
/// @warning 务必记得 `qiniu_ng_str_t` 需要在使用完毕后调用 `qiniu_ng_str_free()` 释放内存。
#[no_mangle]
pub extern "C" fn qiniu_ng_header_info_get_size(header_info: qiniu_ng_header_info_t) -> qiniu_ng_str_t {
    let header_info = Option::<Box<HeaderInfo>>::from(header_info).unwrap();
    unsafe { qiniu_ng_str_t::from_optional_str_unchecked(header_info.size()) }.tap(|_| {
        let _ = qiniu_ng_header_info_t::from(header_info);
    })
}

/// @brief 获取 Header 信息的 Etag 字段
/// @param[in] header_info Header 信息
/// @retval qiniu_ng_str_t 返回 Header 信息的 Etag 字段
/// @note 这里返回的 `qiniu_ng_str_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_is_null()` 进行判断
/// @warning 务必记得 `qiniu_ng_str_t` 需要在使用完毕后调用 `qiniu_ng_str_free()` 释放内存。
#[no_mangle]
pub extern "C" fn qiniu_ng_header_info_get_etag(header_info: qiniu_ng_header_info_t) -> qiniu_ng_str_t {
    let header_info = Option::<Box<HeaderInfo>>::from(header_info).unwrap();
    unsafe { qiniu_ng_str_t::from_optional_str_unchecked(header_info.etag()) }.tap(|_| {
        let _ = qiniu_ng_header_info_t::from(header_info);
    })
}

/// @brief 获取 Header 信息的 Metadata 字段
/// @param[in] header_info Header 信息
/// @retval qiniu_ng_str_t 返回 Header 信息的 Metadata 字段
/// @note 这里返回的 `qiniu_ng_str_map_t` 有可能封装的是 `NULL`，请调用 `qiniu_ng_str_map_is_null()` 进行判断
/// @warning 务必记得 `qiniu_ng_str_map_t` 需要在使用完毕后调用 `qiniu_ng_str_free()` 释放内存。
#[no_mangle]
pub extern "C" fn qiniu_ng_header_info_get_metadata(header_info: qiniu_ng_header_info_t) -> qiniu_ng_str_map_t {
    let header_info = Option::<Box<HeaderInfo>>::from(header_info).unwrap();
    let mut metadata: QiniuNgStrMap = Box::new(HashMap::with_capacity(header_info.metadata().len()));
    for (metadata_key, metadata_value) in header_info.metadata().iter() {
        metadata.insert(
            unsafe { UCString::from_str_unchecked(metadata_key) }.into_boxed_ucstr(),
            unsafe { UCString::from_str_unchecked(metadata_value) }.into_boxed_ucstr(),
        );
    }
    let _ = qiniu_ng_header_info_t::from(header_info);
    metadata.into()
}
