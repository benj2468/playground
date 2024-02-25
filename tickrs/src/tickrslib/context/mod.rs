pub mod pubsub;
pub mod time;

use pubsub::{PubSubSystem, UnderlyingPubSub};

use self::time::TimeSystem;

// Generic Ticker Context
//
// Libraries can implement tickers with this context
pub trait GenericTickerContext {
    type PubSubImpl: UnderlyingPubSub;

    /// The standard Time System
    fn time(&mut self) -> &mut TimeSystem;

    /// The standard pubsub interface that we will consider to be basic across all contexts
    ///
    /// Standard functionality
    fn pubsub(&mut self) -> &mut PubSubSystem<Self::PubSubImpl>;
}
