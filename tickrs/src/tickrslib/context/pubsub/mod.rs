mod test;
pub use test::*;

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

pub trait UnderlyingPubSub {
    fn process(&mut self, data: Vec<Vec<u8>>);

    fn subscribe(&mut self, sub_data: SubData, callback: impl FnMut(Vec<u8>) + 'static);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum SubOrigin {
    Id(u32),
    Unspecified,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct SubData {
    trait_topic: TraitTopic,
    pub origin: SubOrigin,
}

impl SubData {
    pub const fn new<S>(origin: SubOrigin) -> Self
    where
        S: SubscribeTy,
    {
        Self {
            trait_topic: S::TRAIT_TOPIC,
            origin,
        }
    }
}

pub struct Subscription {
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    queue: VecDeque<Vec<u8>>,
}

pub type TraitTopic = (&'static str, u32);

#[derive(Default)]
pub struct PubSubSystem<T> {
    pub_data: Vec<Vec<u8>>,
    sub_data: HashMap<SubData, Subscription>,
    system: T,
}

pub trait SubscribeTy: Sized {
    const TRAIT_TOPIC: TraitTopic;

    fn deserialize(data: &[u8]) -> Result<Self, String>;
}
pub trait PublishTy: Sized + Debug {
    const TRAIT_TOPIC: TraitTopic;

    fn serialize(self) -> Vec<u8>;
}

impl<T> PubSubSystem<T>
where
    T: UnderlyingPubSub,
{
    /// For PubSub Reading Off Queues
    pub fn pop<S>(&mut self, sub: SubData) -> Option<S>
    where
        S: SubscribeTy,
    {
        assert_eq!(S::TRAIT_TOPIC, sub.trait_topic);

        self.sub_data
            .get_mut(&sub)
            .and_then(|subscription| subscription.queue.pop_front())
            .and_then(|front| S::deserialize(&front).ok())
    }

    pub fn peek<S>(&self, sub: SubData) -> Option<S>
    where
        S: SubscribeTy,
    {
        assert_eq!(S::TRAIT_TOPIC, sub.trait_topic);

        self.sub_data
            .get(&sub)
            .and_then(|subscription| subscription.queue.front())
            .and_then(|front| S::deserialize(&front).ok())
    }

    /// For PubSub Writing To Queues
    pub fn publish<P>(&mut self, data: P)
    where
        P: PublishTy,
    {
        println!("Publishing: {:?}", data);
        self.pub_data.push(data.serialize())
    }

    pub fn subscribe(&mut self, sub: SubData) {
        let (tx, rx) = std::sync::mpsc::channel();

        self.system.subscribe(sub, move |data| {
            let _ = tx.send(data);
        });

        self.sub_data.insert(
            sub,
            Subscription {
                rx,
                queue: Default::default(),
            },
        );
    }

    pub(crate) fn process(&mut self) {
        self.system.process(self.pub_data.drain(..).collect());

        for subscription in self.sub_data.values_mut() {
            while let Ok(value) = subscription.rx.try_recv() {
                subscription.queue.push_back(value);
            }
        }
    }
}
