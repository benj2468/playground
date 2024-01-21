use crate::HellowContext;

impl HellowContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn say_hi(&self) {
        println!("Hello {}", self.name);
    }
}
