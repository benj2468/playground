use crate::*;

use self::pubsub::{PublishTy, SubData, SubOrigin, SubscribeTy, TraitTopic};

const HELLO_TRAIT: &str = "hello";
const HELLO_TOPIC: u32 = 1;
const HELLO_TRAIT_TOPIC: (&str, u32) = (HELLO_TRAIT, HELLO_TOPIC);

const SUBSCRIPTION: SubData = SubData::new::<HelloWorld>(SubOrigin::Unspecified);

#[derive(Debug)]
pub struct HelloWorld {
    msg: String,
}

impl SubscribeTy for HelloWorld {
    const TRAIT_TOPIC: TraitTopic = HELLO_TRAIT_TOPIC;

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        let msg = std::str::from_utf8(data).unwrap_or_default().to_string();

        Ok(Self { msg })
    }
}

impl PublishTy for HelloWorld {
    const TRAIT_TOPIC: TraitTopic = HELLO_TRAIT_TOPIC;

    fn serialize(self) -> Vec<u8> {
        self.msg.as_bytes().to_vec()
    }
}

pub struct HelloWorldTicker;

impl<C> crate::Ticker<C> for HelloWorldTicker
where
    C: GenericTickerContext,
{
    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_nanos(1)
    }

    fn init(&mut self, ctx: &mut C) {
        let pubsub = ctx.pubsub();

        pubsub.subscribe(SUBSCRIPTION);
    }

    fn update(&mut self, ctx: &mut C) {
        let pubsub = ctx.pubsub();

        if let Some(hello) = pubsub.pop::<HelloWorld>(SUBSCRIPTION) {
            pubsub.publish(HelloWorld {
                msg: format!("Hello: {}", hello.msg),
            });
        } else {
            pubsub.publish(HelloWorld {
                msg: "World".into(),
            })
        }
    }
}
