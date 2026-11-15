#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    VBlank = 0,
    Timer = 1,
    Input = 2,
}

impl Interrupt {
    pub const ALL: [Interrupt; 3] = [Interrupt::VBlank, Interrupt::Timer, Interrupt::Input];
    pub const MASK_ALL: u8 = 0b0000_0111;

    pub fn bit(&self) -> u8 {
        1 << (*self as u8)
    }

    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Interrupt::VBlank),
            1 => Some(Interrupt::Timer),
            2 => Some(Interrupt::Input),
            _ => None,
        }
    }

    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

pub struct InterruptController {
    enabled: u8,
    requested: u8,

    master: bool,
    vectors: [Option<u16>; 3],
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            enabled: 0,
            requested: 0,

            master: false,
            vectors: [None; 3],
        }
    }

    pub fn enable(&mut self, mask: u8) {
        self.enabled |= mask & Interrupt::MASK_ALL;
    }

    pub fn disable(&mut self, mask: u8) {
        self.enabled &= !(mask & Interrupt::MASK_ALL);
    }

    pub fn set_master(&mut self, enabled: bool) {
        self.master = enabled;
    }

    pub fn set_vector(&mut self, interrupt: Interrupt, vector: u16) {
        self.vectors[interrupt as usize] = Some(vector);
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.requested |= interrupt.bit();
    }

    pub fn has_pending(&self) -> bool {
        (self.requested & self.enabled) != 0
    }

    pub fn has_pending_interrupt(&self, interrupt: Interrupt) -> bool {
        (self.requested & self.enabled & interrupt.bit()) != 0
    }

    pub fn acknowledge_interrupt(&mut self, interrupt: Interrupt) {
        self.requested &= !interrupt.bit();
    }

    pub fn acknowledge(&mut self) -> Option<(Interrupt, u16)> {
        if !self.master {
            return None;
        }

        let pending = self.requested & self.enabled;
        let interrupt = Interrupt::ALL
            .into_iter()
            .find(|i| pending & i.bit() != 0)?;

        self.requested &= !interrupt.bit();
        let vector = self.vectors[interrupt as usize]?;
        self.master = false;

        Some((interrupt, vector))
    }
}
