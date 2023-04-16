use crate::ffi::request_rec;

use crate::core::Pool;

use std::ffi::CStr;

#[repr(transparent)]
pub struct Request(request_rec);

impl Request {
    pub unsafe fn from_request_rec<'a>(r: *mut request_rec) -> &'a mut Request {
        &mut *r.cast::<Request>()
    }

    pub fn is_handler(&self, name: &str) -> bool {
        let handler = unsafe { CStr::from_ptr(self.0.handler) };
        name == handler.to_str().unwrap()
    }

    pub fn pool(&self) -> &mut Pool {
        unsafe { Pool::from_apr_pool_t(self.0.pool) }
    }
}
