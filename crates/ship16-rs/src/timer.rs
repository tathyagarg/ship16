pub struct Timer {
    pub enabled: bool,
    pub periodic: bool,

    pub counter: u16,
    pub reload_to: u16,

    pub prescaler_accumulator: u16,
}

impl Timer {
    const PRESCALER: u16 = 16667;

    pub fn new() -> Self {
        Timer {
            enabled: true,
            periodic: false,
            counter: 0,
            reload_to: 0,
            prescaler_accumulator: 0,
        }
    }

    pub fn set_counter(&mut self, value: u16) {
        self.counter = value;
    }

    pub fn set_reload(&mut self, value: u16) {
        self.reload_to = value;
    }

    pub fn tick_n(&mut self, n: u16) -> bool {
        if !self.enabled {
            return false;
        }

        self.prescaler_accumulator += n;

        let ticks = self.prescaler_accumulator / Self::PRESCALER;
        self.prescaler_accumulator %= Self::PRESCALER;

        if ticks == 0 {
            return false;
        }

        if ticks < self.counter {
            self.counter -= ticks;
            return false;
        }

        if self.periodic {
            let remaining = ticks - self.counter;
            let period = self.reload_to;

            if period == 0 {
                self.counter = 0;
                return true;
            }

            self.counter = period - (remaining % period);
        } else {
            self.counter = 0;
            self.enabled = false;
        }

        true
    }
}
