use libc::c_char;
use std::ffi::CStr;

pub mod rs;

#[derive(Default, Debug)]
pub struct HellowContext {
    name: String,
}

#[no_mangle]
pub extern "C" fn Hellow_new() -> Box<HellowContext> {
    Box::new(HellowContext::new())
}

/// # Safety
///
/// `name` must be a null terminated string that has at most isize::MAX bytes
#[no_mangle]
pub unsafe extern "C" fn Hellow_set_name(ctx: *mut HellowContext, name: *const c_char) -> isize {
    let name = match CStr::from_ptr(name).to_owned().into_string() {
        Err(e) => {
            eprintln!("{e:?}");
            return -1;
        }
        Ok(s) => s,
    };

    let ctx = match ctx.as_mut() {
        None => {
            eprintln!("Cannot set name for null context");
            return -1;
        }
        Some(ctx) => ctx,
    };

    ctx.name = name;

    0
}

#[no_mangle]
pub extern "C" fn Hellow_say_hi(ctx: &HellowContext) {
    ctx.say_hi()
}
