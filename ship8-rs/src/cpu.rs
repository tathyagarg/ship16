use crate::{constants::*, memory::Memory};

pub const REGISTER_COUNT: usize = 16;

pub enum StackFault {
    Overflow,
    Underflow,
}

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

    pub fn push_u16(&mut self, mem: &mut Memory, value: u16) -> Result<(), StackFault> {
        if self.sp <= STACK_LIMIT as u16 {
            return Err(StackFault::Overflow);
        }

        mem.write_byte(self.sp, (value & 0xFF) as u8);
        self.sp = self.sp.wrapping_sub(1);
        mem.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        Ok(())
    }

    pub fn pop_u16(&mut self, mem: &Memory) -> Result<u16, StackFault> {
        if self.sp > STACK_TOP - 2 {
            return Err(StackFault::Underflow);
        }

        self.sp = self.sp.wrapping_add(1);
        let low = mem.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = mem.read_byte(self.sp) as u16;
        Ok((high << 8) | low)
    }
}
