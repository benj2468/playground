pub type CResult = Result<(), String>;

pub fn check_null<'a, T>(value: *mut T) -> Result<&'a mut T, &'static str> {
    if value.is_null() {
        Err("Null Pointed Received by Rust")
    } else {
        Ok(unsafe { &mut *value })
    }
}

pub fn as_string(c_str: *const libc::c_char) -> Result<String, String> {
    unsafe { std::ffi::CStr::from_ptr(c_str) }
        .to_str()
        .map_err(|e| format!("{e:?}"))
        .map(|s| s.to_string())
}
