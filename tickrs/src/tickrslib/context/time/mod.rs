/// All units are in microseconds
pub struct TimeSystem {
    /// The time we use to tick the real clock
    tick_rate: std::time::Duration,
    /// The simulated time used to tick the systems
    true_tick_rate: std::time::Duration,
    tick_time: u128,
}

impl TimeSystem {
    pub fn new(tick_rate: std::time::Duration) -> Self {
        Self {
            tick_rate,
            true_tick_rate: tick_rate,
            tick_time: 0,
        }
    }

    pub fn new_ftrt(tick_rate: std::time::Duration, true_rate: std::time::Duration) -> Self {
        Self {
            tick_rate,
            true_tick_rate: true_rate,
            tick_time: 0,
        }
    }

    pub fn sleep(&self) {
        std::thread::sleep(self.tick_rate);
    }

    pub fn tick(&mut self) {
        self.tick_time += self.true_tick_rate.as_micros();
    }

    pub fn get_time(&self) -> u128 {
        self.tick_time
    }
}
