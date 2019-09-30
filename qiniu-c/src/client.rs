use crate::config::qiniu_ng_config_t;
use libc::{c_char, c_void};
use qiniu::Client;
use std::{ffi::CStr, mem};

#[repr(C)]
pub struct qiniu_ng_client_t(*mut c_void);

impl From<qiniu_ng_client_t> for Box<Client> {
    fn from(client: qiniu_ng_client_t) -> Self {
        unsafe { Box::from_raw(mem::transmute::<_, *mut Client>(client)) }
    }
}

impl From<Box<Client>> for qiniu_ng_client_t {
    fn from(client: Box<Client>) -> Self {
        unsafe { mem::transmute(Box::into_raw(client)) }
    }
}

#[no_mangle]
pub extern "C" fn qiniu_ng_client_new(
    access_key: *const c_char,
    secret_key: *const c_char,
    config: *const qiniu_ng_config_t,
) -> qiniu_ng_client_t {
    Box::new(Client::new(
        unsafe { CStr::from_ptr(access_key).to_string_lossy() },
        unsafe { CStr::from_ptr(secret_key).to_string_lossy() },
        unsafe { config.as_ref() }.unwrap().into(),
    ))
    .into()
}

#[no_mangle]
pub extern "C" fn qiniu_ng_client_free(client: qiniu_ng_client_t) {
    let _: Box<Client> = client.into();
}
