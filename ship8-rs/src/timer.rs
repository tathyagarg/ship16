pub struct Timer {
    pub enabled: bool,
    pub periodic: bool,

    pub counter: u16,
    pub reload_to: u16,

    pub prescaler_accumulator: u16,
}

impl Timer {
    const PRESCALER: u16 = 1024;

    pub fn new() -> Self {
        Timer {
            enabled: true,
            periodic: false,
            counter: 0,
            reload_to: 0,
            prescaler_accumulator: 0,
        }
    }

    pub fn tick_n(&mut self, n: u16) -> bool {
        if !self.enabled {
            return false;
        }

        self.prescaler_accumulator += n;

        if self.prescaler_accumulator >= Self::PRESCALER {
            let ticks = self.prescaler_accumulator / Self::PRESCALER;
            self.prescaler_accumulator %= Self::PRESCALER;

            if self.counter > ticks {
                self.counter -= ticks;
            } else {
                if self.periodic {
                    self.counter = self.reload_to;
                }
                return true; // Timer has expired
            }
        }

        false
    }
}
