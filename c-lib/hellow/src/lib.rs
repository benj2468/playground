#[derive(Default)]
pub struct HellowContext {
    name: String,
}

#[no_mangle]
pub unsafe extern "C" fn new_hellow() -> Box<HellowContext> {
    let ctx = HellowContext {
        name: "Benjamin Cape".into(),
    };

    Box::new(ctx)
}

#[no_mangle]
pub unsafe extern "C" fn say_hi(ctx: &HellowContext) {
    println!("Hello {:?}", ctx.name);
}
