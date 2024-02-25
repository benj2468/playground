use tickrs::{
    pubsub::{PubSubSystem, TestUnderlyingPubSub},
    tickers::HelloWorldTicker,
    time::TimeSystem,
    *,
};

pub struct CustomTicker;

impl Ticker<MyFirstContext> for CustomTicker {
    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    fn update(&mut self, ctx: &mut MyFirstContext) {
        ctx.health.test_caller()
    }
}

pub struct TelemetrySystem;

impl TelemetrySystem {
    fn update(&mut self) {}

    fn test_caller(&self) {
        println!("Called the health service");
    }
}

macro_rules! tickrs {
    ($name:ident {
        cfg {
            PubSub = $pub_sub_system:ty,
        }
        $($field:ident: $ty:ty),*
    }) => {
        pub struct $name {
            time: TimeSystem,
            pubsub: ::tickrs::pubsub::PubSubSystem<$pub_sub_system>,
            $($field: $ty),*
        }

        impl GenericTickerContext for $name {
            type PubSubImpl = $pub_sub_system;

            fn time(&mut self) -> &mut TimeSystem {
                &mut self.time
            }
            fn pubsub(&mut self) -> &mut PubSubSystem<$pub_sub_system> {
                &mut self.pubsub
            }
        }
    };
}

tickrs!(MyFirstContext {
    cfg {
        PubSub = TestUnderlyingPubSub,
    }
    health: TelemetrySystem
});

fn main() {
    let ctx = MyFirstContext {
        time: TimeSystem::new(std::time::Duration::from_secs(1)),
        pubsub: Default::default(),
        health: TelemetrySystem,
    };

    let me = MasterTicker::new(ctx)
        .with_system(HelloWorldTicker)
        .with_system(CustomTicker)
        .with_pre_pubsub(|c| {
            c.health.update();
        })
        .init();

    me.run();
}
