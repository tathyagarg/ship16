use crate::constants::CYCLES_PER_FRAME;

pub struct Ppu {
    pub x: u8,
    pub y: u8,

    pub counter: u64,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            x: 0,
            y: 0,
            counter: 0,
        }
    }

    pub fn move_to_decode(&mut self, pos: u16) {
        self.x = ((pos >> 6) & 0x3F) as u8;
        self.y = (pos & 0x3F) as u8;
    }

    pub fn tick_n(&mut self, n: u64) -> bool {
        self.counter += n;
        if self.counter >= CYCLES_PER_FRAME {
            self.counter %= CYCLES_PER_FRAME;
            return true;
        }
        false
    }
}
