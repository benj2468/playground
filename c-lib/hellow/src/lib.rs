use libc::c_char;

mod unsafe_work {
    use std::ffi::CStr;

    use libc::c_char;

    pub unsafe fn extract_char_ptr(c: *const c_char) -> String {
        CStr::from_ptr(c).to_owned().into_string().unwrap()
    }
}

#[derive(Default, Debug)]
pub struct HellowContext {
    name: String,
}

#[no_mangle]
pub extern "C" fn Hellow_new() -> Box<HellowContext> {
    Default::default()
}

#[no_mangle]
pub unsafe extern "C" fn Hellow_set_name(ctx: *mut HellowContext, name: *const c_char) -> isize {
    if ctx.is_null() {
        eprintln!("Cannot set name for null context");
        return -1;
    }

    let name = unsafe_work::extract_char_ptr(name);

    (*ctx).name = name;

    0
}

#[no_mangle]
pub extern "C" fn Hellow_say_hi(ctx: &HellowContext) {
    println!("Hello {}", ctx.name);
}
