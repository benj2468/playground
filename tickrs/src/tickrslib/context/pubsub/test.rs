use crate::pubsub::{SubData, UnderlyingPubSub};

#[derive(Default)]
pub struct TestUnderlyingPubSub {
    callbacks: Vec<Box<dyn FnMut(Vec<u8>)>>,
}

impl UnderlyingPubSub for TestUnderlyingPubSub {
    fn process(&mut self, data: Vec<Vec<u8>>) {
        for callback in &mut self.callbacks {
            for msg in data.clone() {
                (callback)(msg)
            }
        }
    }

    fn subscribe(&mut self, _sub_data: SubData, callback: impl FnMut(Vec<u8>) + 'static) {
        self.callbacks.push(Box::new(callback))
    }
}