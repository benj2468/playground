use pyo3::prelude::*;

#[pyclass(subclass)]
struct Cortex {
    tx: std::sync::mpsc::Sender<u32>,
    rx: std::sync::Arc<std::sync::Mutex<u32>>,
    _handles: Vec<std::thread::JoinHandle<()>>,
}

#[pymethods]
impl Cortex {
    #[new]
    fn new() -> Self {
        let value = std::sync::Arc::new(std::sync::Mutex::new(0));
        let (tx, rx) = std::sync::mpsc::channel();

        let mut _handles = vec![];

        _handles.push(std::thread::spawn({
            let value = value.clone();

            move || {
                while let Ok(new) = rx.recv() {
                    *value.lock().unwrap() = new;
                }
            }
        }));

        _handles.push(std::thread::spawn({
            let value = value.clone();
            move || loop {
                *value.lock().unwrap() += 10;
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
            }
        }));

        Self {
            tx,
            rx: value,
            _handles,
        }
    }

    pub fn get(&self) -> u32 {
        *self.rx.lock().unwrap()
    }

    pub fn tick(&self, new_value: u32) {
        self.tx.send(new_value).unwrap();
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn cortex(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Cortex>()?;
    Ok(())
}
