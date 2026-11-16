use isa::constants::CYCLES_PER_FRAME;

#[derive(Default)]
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

    pub fn tick_n(&mut self, n: u64) -> bool {
        self.counter += n;
        if self.counter >= CYCLES_PER_FRAME {
            self.counter %= CYCLES_PER_FRAME;
            return true;
        }
        false
    }

    pub fn clear(&mut self, buffer: &mut [u8]) {
        for pixel in buffer.iter_mut() {
            *pixel = 0;
        }
    }

    pub fn draw_pixel(&mut self, buffer: &mut [u8], color: u8) {
        let index = (self.y as usize * 64 + self.x as usize) % buffer.len();
        buffer[index] = color;
    }

    pub fn set_cursor_packed(&mut self, pos: u16) {
        self.x = ((pos >> 6) & 0x3F) as u8;
        self.y = (pos & 0x3F) as u8;
    }

    pub fn move_x(&mut self, delta: i16) {
        let new_x = self.x as i16 + delta;
        self.x = (new_x as u8) % 64;
    }

    pub fn move_y(&mut self, delta: i16) {
        let new_y = self.y as i16 + delta;
        self.y = (new_y as u8) % 64;
    }

    pub fn set_x(&mut self, x: u16) {
        self.x = (x as u8).min(63);
    }

    pub fn set_y(&mut self, y: u16) {
        self.y = (y as u8).min(63);
    }
}
