#[macro_export]
macro_rules! hook_handler {
    ( $name: ident, $handler: expr ) => {
        #[no_mangle]
        extern "C" fn $name(r: *mut request_rec) -> c_int {
            $handler(unsafe { &mut $crate::httpd::Request::from_request_rec(r) })
        }
    };
}

#[macro_export]
macro_rules! hook_post_read_request {
    ( $name: ident, $handler: expr ) => {
        #[no_mangle]
        extern "C" fn $name(r: *mut request_rec) -> c_int {
            $handler(unsafe { &mut $crate::httpd::Request::from_request_rec(r) })
        }
    };
}
