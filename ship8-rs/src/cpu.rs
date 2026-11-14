pub const REGISTER_COUNT: usize = 16;

pub struct Cpu {
    pub registers: [u16; REGISTER_COUNT],
    pub pc: u16,
    pub sp: u16,
    pub index: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; REGISTER_COUNT],
            pc: 0x0900,
            sp: 0xFFFF,
            index: 0x0200,
        }
    }
}
