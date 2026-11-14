pub struct Input(u16);

impl Input {
    pub fn new() -> Self {
        Input(0)
    }

    pub fn key_down(&mut self, key: u8) -> bool {
        let mask = 1u16 << key;
        let newly_pressed = self.0 & mask == 0;

        self.0 |= mask;
        newly_pressed
    }

    pub fn key_up(&mut self, key: u8) {
        let mask = 1u16 << key;
        self.0 &= !mask;
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        assert!(key < 16, "Key index out of bounds");

        let mask = 1u16 << key;
        self.0 & mask != 0
    }
}
