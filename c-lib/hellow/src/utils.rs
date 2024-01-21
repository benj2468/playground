use std::ffi::IntoStringError;

pub unsafe fn parse_c_str(s: *const libc::c_char) -> std::result::Result<String, IntoStringError> {
    std::ffi::CStr::from_ptr(s).to_owned().into_string()
}

#[macro_export]
macro_rules! try_parse_c_str {
    ($str:ident) => {
        match crate::utils::parse_c_str($str) {
            Err(e) => {
                eprintln!("{:?}", e);
                return -1;
            }
            Ok(s) => s,
        }
    };
}
