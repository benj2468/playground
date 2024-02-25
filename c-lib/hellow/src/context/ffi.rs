use crate::{context::HellowContext, try_parse_c_str};
use libc::c_char;

#[no_mangle]
pub extern "C" fn Hellow_new() -> Box<HellowContext> {
    Box::new(HellowContext::new())
}

/// # Safety
///
/// `name` must be a null terminated string that has at most isize::MAX bytes
#[no_mangle]
pub unsafe extern "C" fn Hellow_set_prefix(
    ctx: *mut HellowContext,
    prefix: *const c_char,
) -> isize {
    let prefix = try_parse_c_str!(prefix);

    let ctx = match ctx.as_mut() {
        None => {
            eprintln!("Cannot set name for null context");
            return -1;
        }
        Some(ctx) => ctx,
    };

    ctx.prefix = prefix;

    0
}

#[no_mangle]
pub unsafe extern "C" fn Hellow_announce(ctx: &HellowContext, name: *const c_char) -> isize {
    let name = try_parse_c_str!(name);

    ctx.announce(name);

    0
}
