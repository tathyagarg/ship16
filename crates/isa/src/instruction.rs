use crate::interrupt::Interrupt;

pub const CLEAR_CYCLES: u32 = 4097;
pub const SPRITE_CYCLES: u32 = 66;

pub type RegisterIndex = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Zero,
    Carry,
    Overflow,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Cond {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Cond::Eq),
            1 => Some(Cond::Ne),
            2 => Some(Cond::Lt),
            3 => Some(Cond::Gt),
            4 => Some(Cond::Le),
            5 => Some(Cond::Ge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AluOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Rol,
    Ror,
}

impl AluOp {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(AluOp::Add),
            1 => Some(AluOp::Sub),
            2 => Some(AluOp::Mul),
            3 => Some(AluOp::Div),
            4 => Some(AluOp::Rem),
            5 => Some(AluOp::And),
            6 => Some(AluOp::Or),
            7 => Some(AluOp::Xor),
            8 => Some(AluOp::Not),
            9 => Some(AluOp::Shl),
            10 => Some(AluOp::Shr),
            11 => Some(AluOp::Rol),
            12 => Some(AluOp::Ror),
            _ => None,
        }
    }

    pub fn cycles(&self) -> u32 {
        match self {
            AluOp::Mul => 3,
            AluOp::Div | AluOp::Rem => 5,
            AluOp::Shl | AluOp::Shr | AluOp::Rol | AluOp::Ror => 2,
            _ => 1,
        }
    }
}

pub struct AluOut {
    pub result: u16,
    pub carry: bool,
    pub overflow: bool,
}

#[derive(Debug)]
pub enum Instr {
    Nop,
    Halt,
    Wait,

    Load8(RegisterIndex),
    Store8(RegisterIndex),
    Load16(RegisterIndex),
    Store16(RegisterIndex),
    AddIndex(RegisterIndex),
    Push(RegisterIndex),
    Pop(RegisterIndex),
    SkipIfFlag {
        flag: Flag,
        set: bool,
    },

    TimerCount(RegisterIndex),
    TimerReload(RegisterIndex),
    TimerRead(RegisterIndex),
    TimerConfig {
        enable: bool,
        periodic: bool,
    },

    IntEnable(u8),
    IntDisable(u8),
    SetVector(Interrupt),
    Ei,
    Di,
    Reti,

    Clear,
    DrawColor(u8),
    DrawColorReg(RegisterIndex),
    DrawSprite(u8),
    DrawSpriteReg(RegisterIndex),
    MoveTo(u16),
    AddX(u8),
    AddY(u8),
    SubX(u8),
    SubY(u8),
    SetXReg(RegisterIndex),
    SetYReg(RegisterIndex),

    Jump(u16),
    JumpLong(u16),
    Call(u16),
    CallLong(u16),
    Return,
    SkipIf {
        cond: Cond,
        reg_x: RegisterIndex,
        reg_y: RegisterIndex,
    },
    SkipIfImm {
        cond: Cond,
        reg_x: RegisterIndex,
        imm: u16,
    },
    SkipIfKey(u8),

    SetReg {
        reg: RegisterIndex,
        value: u8,
    },
    SetRegLong {
        reg: RegisterIndex,
        value: u16,
    },
    Copy {
        reg_x: RegisterIndex,
        reg_y: RegisterIndex,
    },
    Alu {
        op: AluOp,
        reg_x: RegisterIndex,
        reg_y: RegisterIndex,
    },
    AluImm {
        op: AluOp,
        reg_x: RegisterIndex,
        imm: u16,
    },
    Random(RegisterIndex),
    SetIndex(u16),
    SetIndexLong(u16),
}

impl Instr {
    pub fn base_cycles(&self) -> u32 {
        match self {
            Self::Nop | Self::Copy { .. } | Self::Halt => 1,
            Self::Clear => CLEAR_CYCLES,
            Self::DrawSprite(_) | Self::DrawSpriteReg(_) => SPRITE_CYCLES,
            Self::Call(_) | Self::Return => 4,
            Self::SkipIf { .. } | Self::SkipIfKey(_) => 2,
            Self::Alu { op, .. } => op.cycles(),
            _ => 2,
        }
    }
}

#[derive(Debug)]
pub struct Illegal(u16);

impl std::fmt::Display for Illegal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Illegal instruction: 0x{:04X}", self.0)
    }
}

pub fn decode(raw: u16, extra: u16) -> Result<Instr, Illegal> {
    let nib = |shift: u32| ((raw >> shift) & 0xF) as RegisterIndex;
    let imm8 = (raw & 0x00FF) as u8;
    let addr = raw & 0x0FFF;
    let bad = Illegal(raw);

    Ok(match raw {
        0x0000 => Instr::Nop,

        0x0200..=0x020F => Instr::Load8(nib(0)),
        0x0210..=0x021F => Instr::Store8(nib(0)),
        0x0220..=0x022F => Instr::Load16(nib(0)),
        0x0230..=0x023F => Instr::Store16(nib(0)),
        0x0240..=0x024F => Instr::AddIndex(nib(0)),
        0x0300..=0x030F => Instr::Push(nib(0)),
        0x0310..=0x031F => Instr::Pop(nib(0)),
        0x0400..=0x041F => Instr::SkipIfFlag {
            flag: match nib(0) {
                0 => Flag::Zero,
                1 => Flag::Carry,
                2 => Flag::Overflow,
                3 => Flag::Negative,
                _ => return Err(bad),
            },
            set: (raw & 0x0080) != 0,
        },

        0x01000 => Instr::Clear,

        0x1100..=0x110F => Instr::TimerCount(nib(0)),
        0x1110..=0x111F => Instr::TimerReload(nib(0)),
        0x1120..=0x112F => Instr::TimerRead(nib(0)),
        0x1130..=0x1133 => Instr::TimerConfig {
            enable: (raw & 0x0002) != 0,
            periodic: (raw & 0x0001) != 0,
        },
        0x1200..=0x120F => Instr::IntEnable(nib(0) as u8),
        0x1210..=0x121F => Instr::IntDisable(nib(0) as u8),
        0x1300..=0x130F => Instr::SetVector(Interrupt::from_index(nib(0) as u8).ok_or(bad)?),
        0x1400 => Instr::Ei,
        0x1401 => Instr::Di,

        0x1500 => Instr::Wait,

        0x2000..=0x20FF => Instr::DrawColor(imm8),
        0x2100..=0x210F => Instr::DrawColorReg(nib(0)),
        0x2200..=0x22FF => Instr::DrawSprite(imm8),
        0x2300..=0x230F => Instr::DrawSpriteReg(nib(0)),

        0x3000..=0x3FFF => Instr::MoveTo(addr),
        0x4000..=0x43FF => match nib(8) {
            0 => Instr::AddX(imm8),
            1 => Instr::AddY(imm8),
            2 => Instr::SubX(imm8),
            3 => Instr::SubY(imm8),
            _ => return Err(bad),
        },
        0x4A00..=0x4A0F => Instr::SetXReg(nib(0)),
        0x4B00..=0x4B0F => Instr::SetYReg(nib(0)),

        0x5000..=0x5FFF => Instr::Jump(addr),
        0x6000..=0x6FFF => Instr::SkipIf {
            cond: Cond::from_u8(nib(8) as u8).ok_or(bad)?,
            reg_x: nib(4),
            reg_y: nib(0),
        },
        0x7000..=0x7FFF => Instr::SetReg {
            reg: nib(8),
            value: imm8,
        },
        0x8000..=0x80FF => Instr::Copy {
            reg_x: nib(4),
            reg_y: nib(0),
        },
        0x9000..=0x9FFF => Instr::Call(addr),

        0xA000 => Instr::Return,
        0xA001 => Instr::Reti,

        0xB000..=0xB00F => Instr::SkipIfKey(nib(0) as u8),
        0xC000..=0xCFFF => Instr::Alu {
            op: AluOp::from_u8(nib(8) as u8).ok_or(bad)?,
            reg_x: nib(4),
            reg_y: nib(0),
        },
        0xD000..=0xD00F => Instr::Random(nib(0)),
        0xE000..=0xEFFF => Instr::SetIndex(addr),

        0xF000 => Instr::JumpLong(extra),
        0xF100 => Instr::CallLong(extra),
        0xF200 => Instr::SetIndexLong(extra),
        0xF300..=0xF30F => Instr::SetRegLong {
            reg: nib(0),
            value: extra,
        },

        0xF400..=0xF4FF => Instr::AluImm {
            op: AluOp::from_u8(nib(4) as u8).ok_or(bad)?,
            reg_x: nib(0),
            imm: extra,
        },
        0xF500..=0xF5FF => Instr::SkipIfImm {
            cond: Cond::from_u8(nib(4) as u8).ok_or(bad)?,
            reg_x: nib(0),
            imm: extra,
        },

        0xFFFF => Instr::Halt,
        _ => return Err(bad),
    })
}
