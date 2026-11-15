use crate::memory::Memory;
use isa::{
    constants::*,
    instruction::{AluOut, Flag},
};

pub const REGISTER_COUNT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackFault {
    Overflow,
    Underflow,
}

pub struct Flags {
    pub zero: bool,
    pub carry: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            zero: false,
            carry: false,
            overflow: false,
            negative: false,
        }
    }

    pub fn get(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero => self.zero,
            Flag::Carry => self.carry,
            Flag::Overflow => self.overflow,
            Flag::Negative => self.negative,
        }
    }

    pub fn set_from_alu(&mut self, alu_out: &AluOut) {
        self.zero = alu_out.result == 0;
        self.carry = alu_out.carry;
        self.overflow = alu_out.overflow;
        self.negative = (alu_out.result & 0x8000) != 0;
    }
}

pub struct Cpu {
    pub registers: [u16; REGISTER_COUNT],
    pub pc: u16,
    pub sp: u16,
    pub index: u16,
    pub waiting: bool,
    pub flags: Flags,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; REGISTER_COUNT],
            pc: ENTRY_POINT,
            sp: 0xFFFF,
            index: 0x0200,
            waiting: false,
            flags: Flags::new(),
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
        let high = mem.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let low = mem.read_byte(self.sp) as u16;
        Ok((high << 8) | low)
    }
}
