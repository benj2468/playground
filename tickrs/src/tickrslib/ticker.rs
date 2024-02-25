pub trait Ticker<C>
where
    C: crate::GenericTickerContext,
{
    /// The time interval that your ticker will get called at
    fn interval(&self) -> std::time::Duration;

    /// Called once at the beginning of the tickers life
    ///
    /// May not do anything
    fn init(&mut self, _ctx: &mut C) {}

    /// The update function for your ticker
    fn update(&mut self, ctx: &mut C);
}

pub struct TickerManager<C> {
    /// Last tick timestep in microseconds
    last_tick: Option<u128>,
    inner: Box<dyn Ticker<C>>,
}

impl<C> Ticker<C> for TickerManager<C>
where
    C: crate::GenericTickerContext,
{
    fn interval(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    fn init(&mut self, ctx: &mut C) {
        self.inner.init(ctx);
    }

    fn update(&mut self, ctx: &mut C) {
        let duration_micro = self.inner.interval().as_micros();

        println!(
            "[last tick: {}, , duration: {}, current time: {}, next_tick: {}",
            self.last_tick.unwrap_or_default(),
            duration_micro,
            ctx.time().get_time(),
            self.last_tick.unwrap_or_default() + duration_micro
        );

        if self
            .last_tick
            .map(|last_tick| last_tick + duration_micro <= ctx.time().get_time())
            .unwrap_or(true)
        {
            self.inner.update(ctx);
            self.last_tick.replace(ctx.time().get_time());
        }
    }
}

pub struct MasterTicker<C>
where
    C: crate::GenericTickerContext,
{
    context: C,
    systems: Vec<TickerManager<C>>,
    pre_pubsub: Box<dyn Fn(&mut C)>,
    post_pubsub: Box<dyn Fn(&mut C)>,
}

impl<C> MasterTicker<C>
where
    C: crate::GenericTickerContext + 'static,
{
    pub fn new(context: C) -> Self {
        Self {
            context,
            systems: Default::default(),
            pre_pubsub: Box::new(|_| {}),
            post_pubsub: Box::new(|_| {}),
        }
    }

    pub fn with_system(mut self, ticker: impl Ticker<C> + 'static) -> Self {
        self.systems.push(TickerManager {
            last_tick: None,
            inner: Box::new(ticker),
        });

        self
    }

    pub fn with_pre_pubsub(mut self, caller: impl Fn(&mut C) + 'static) -> Self {
        let existing = self.pre_pubsub;

        self.pre_pubsub = Box::new(move |c| {
            existing(c);
            caller(c);
        });

        self
    }

    pub fn with_post_pubsub(mut self, caller: impl Fn(&mut C) + 'static) -> Self {
        let existing = self.post_pubsub;

        self.post_pubsub = Box::new(move |c| {
            existing(c);
            caller(c);
        });

        self
    }

    /// Tick, with n microseconds having elapsed
    pub fn tick(&mut self) {
        for system in &mut self.systems {
            system.update(&mut self.context)
        }

        (self.pre_pubsub)(&mut self.context);

        self.context.pubsub().process();

        (self.post_pubsub)(&mut self.context);
    }

    /// Initialize all the systems
    pub fn init(mut self) -> Self {
        for system in &mut self.systems {
            system.init(&mut self.context)
        }

        self
    }

    /// This is the realtime run function
    pub fn run(mut self) {
        self = self.init();
        loop {
            println!("-------------");
            self.tick();

            self.context.time().sleep();
            self.context.time().tick();
        }
    }
}
