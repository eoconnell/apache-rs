use crate::ffi::apr_palloc;
use crate::ffi::apr_pool_t;
use crate::ffi::apr_pool_userdata_get;
use crate::ffi::apr_pool_userdata_setn;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;

#[repr(transparent)]
pub struct Pool(apr_pool_t);

impl Pool {
    pub unsafe fn from_apr_pool_t<'a>(p: *mut apr_pool_t) -> &'a mut Pool {
        &mut *p.cast::<Pool>()
    }

    fn _alloc(&mut self, size: usize) -> *mut c_void {
        unsafe { apr_palloc(&mut self.0, size) }
    }

    pub fn alloc<T>(&mut self) -> *mut T {
        self._alloc(mem::size_of::<T>()) as *mut T
    }

    pub fn set_userdata<T>(&mut self, data: *mut T, key: &str) {
        let c_key = CString::new(key).unwrap().into_raw();
        unsafe { apr_pool_userdata_setn(data as *const c_void, c_key, None, &mut self.0) };
    }

    pub fn get_userdata<T>(&mut self, key: &str) -> *mut T {
        let mut data: *mut T = std::ptr::null_mut();
        let c_data = &mut data as *mut *mut _ as *mut *mut c_void;
        let c_key = CString::new(key).unwrap().into_raw();
        unsafe { apr_pool_userdata_get(c_data, c_key, &mut self.0) };
        data
    }
}
