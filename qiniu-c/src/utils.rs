use cfg_if::cfg_if;
use libc::{c_char, c_void, size_t};
use std::{boxed::Box, ffi::CString, mem, path::PathBuf, slice};

#[repr(C)]
pub struct qiniu_ng_string_t(*mut c_char);

impl From<CString> for qiniu_ng_string_t {
    fn from(s: CString) -> Self {
        unsafe { mem::transmute(s.into_raw()) }
    }
}

impl From<qiniu_ng_string_t> for CString {
    fn from(s: qiniu_ng_string_t) -> Self {
        unsafe { CString::from_raw(mem::transmute(s)) }
    }
}

pub(crate) fn make_string<S: AsRef<str>>(s: S) -> qiniu_ng_string_t {
    CString::new(s.as_ref()).unwrap().into()
}

#[no_mangle]
pub extern "C" fn qiniu_ng_string_get_ptr(s: qiniu_ng_string_t) -> *const c_char {
    s.0
}

#[no_mangle]
pub extern "C" fn qiniu_ng_string_free(s: qiniu_ng_string_t) {
    let _: CString = s.into();
}

#[repr(C)]
pub struct qiniu_ng_string_list_t(*mut c_void, *mut c_void);

impl From<Box<[CString]>> for qiniu_ng_string_list_t {
    fn from(strlist: Box<[CString]>) -> Self {
        unsafe { mem::transmute(Box::into_raw(strlist)) }
    }
}

impl From<qiniu_ng_string_list_t> for Box<[CString]> {
    fn from(strlist: qiniu_ng_string_list_t) -> Self {
        unsafe { Box::from_raw(mem::transmute(strlist)) }
    }
}

pub(crate) fn make_string_list<S: AsRef<str>, A: AsRef<[S]>>(list: A) -> qiniu_ng_string_list_t {
    list.as_ref()
        .into_iter()
        .map(|s| CString::new(s.as_ref()).unwrap())
        .collect::<Box<[CString]>>()
        .into()
}

#[no_mangle]
pub extern "C" fn qiniu_ng_string_list_len(strlist: qiniu_ng_string_list_t) -> size_t {
    let strlist: Box<[CString]> = strlist.into();
    let len = strlist.len();
    let _: qiniu_ng_string_list_t = strlist.into();
    len
}

#[no_mangle]
pub extern "C" fn qiniu_ng_string_list_get(
    strlist: qiniu_ng_string_list_t,
    index: size_t,
    str_ptr: *mut *const c_char,
) -> bool {
    let strlist: Box<[CString]> = strlist.into();
    let mut got = false;
    if let Some(s) = strlist.get(index) {
        if !str_ptr.is_null() {
            unsafe { *str_ptr = s.as_ptr() };
        }
        got = true;
    }
    let _: qiniu_ng_string_list_t = strlist.into();
    got
}

#[no_mangle]
pub extern "C" fn qiniu_ng_string_list_free(strlist: qiniu_ng_string_list_t) {
    let _: Box<[CString]> = strlist.into();
}

pub(crate) fn write_string_to_ptr<S: AsRef<str>>(src: S, dst: *mut c_char) {
    let src_bytes = src.as_ref();
    unsafe {
        dst.copy_from_nonoverlapping(mem::transmute(src_bytes.as_ptr()), src_bytes.len());
    }
}

cfg_if! {
    if #[cfg(unix)] {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        pub fn make_path_buf(path: *const u8, path_len: size_t) -> PathBuf {
            let buf = unsafe { slice::from_raw_parts(path, path_len) };
            PathBuf::from(OsStr::from_bytes(buf))
        }
    } else if #[cfg(windows)] {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        pub fn make_path_buf(path: *const u8, path_len: size_t) -> PathBuf {
            let buf = unsafe { slice::from_raw_parts(path, path_len) };
            PathBuf::from(OsStr::from_wide(buf))
        }
    } else {
        panic!("Unsupported platform");
    }
}
