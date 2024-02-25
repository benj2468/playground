use std::thread::JoinHandle;

pub mod ffi;

// #[derive(Debug)]
pub struct HellowContext {
    prefix: String,
    _handle: JoinHandle<()>,
    tx: std::sync::mpsc::Sender<String>,
}

impl HellowContext {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let _handle = std::thread::spawn(move || {
            while let Ok(value) = rx.recv() {
                println!("{value}");
            }
        });

        Self {
            prefix: "Hellow ".into(),
            tx,
            _handle,
        }
    }

    pub fn announce(&self, name: String) {
        let _ = self.tx.send(format!("{} {name}", self.prefix));
    }
}
