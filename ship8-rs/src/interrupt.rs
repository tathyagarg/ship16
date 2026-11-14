pub const INTERRUPT_VBLANK: u8 = 0x01;
pub const INTERRUPT_TIMER: u8 = 0x02;
pub const INTERRUPT_INPUT: u8 = 0x04;

pub struct InterruptController {
    enabled: u8,
    requested: u8,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            enabled: 0xFF,
            requested: 0,
        }
    }

    pub fn request_interrupt(&mut self, interrupt: u8) {
        self.requested |= interrupt;
    }

    pub fn clear_interrupt(&mut self, interrupt: u8) {
        self.requested &= !interrupt;
    }

    pub fn is_any_interrupt_pending(&self) -> bool {
        (self.requested & self.enabled) != 0
    }

    pub fn get_pending_interrupt(&self) -> Option<u8> {
        let pending = self.requested & self.enabled;
        if pending == 0 {
            None
        } else {
            Some(pending.trailing_zeros() as u8)
        }
    }
}
