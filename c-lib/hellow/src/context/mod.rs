pub mod ffi;

#[derive(Default, Debug)]
pub struct HellowContext {
    name: String,
}

impl HellowContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn say_hi(&self) {
        println!("Hello {}", self.name);
    }
}
