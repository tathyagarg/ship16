pub const CLEAR_CYCLES: u32 = 4097;
pub const SPRITE_CYCLES: u32 = 66;

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

    pub fn eval(&self, a: u16, b: u16) -> bool {
        match self {
            Cond::Eq => a == b,
            Cond::Ne => a != b,
            Cond::Lt => a < b,
            Cond::Gt => a > b,
            Cond::Le => a <= b,
            Cond::Ge => a >= b,
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

    pub fn eval(&self, a: u16, b: u16) -> u16 {
        match self {
            AluOp::Add => a.wrapping_add(b),
            AluOp::Sub => a.wrapping_sub(b),
            AluOp::Mul => a.wrapping_mul(b),
            AluOp::Div => a / b,
            AluOp::Rem => a % b,
            AluOp::And => a & b,
            AluOp::Or => a | b,
            AluOp::Xor => a ^ b,
            AluOp::Not => !a,
            AluOp::Shl => a << b,
            AluOp::Shr => a >> b,
            AluOp::Rol => a.rotate_left(b as u32),
            AluOp::Ror => a.rotate_right(b as u32),
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

#[derive(Debug)]
pub enum Instr {
    Nop,
    Clear,
    DrawColor(u8),
    DrawColorReg(u8),
    DrawSprite(u8),
    DrawSpriteReg(u8),
    MoveTo(u16),
    AddX(u8),
    AddY(u8),
    SubX(u8),
    SubY(u8),
    SetXReg(u8),
    SetYReg(u8),
    Jump(u16),
    SkipIf { cond: Cond, reg_x: u8, reg_y: u8 },
    SetReg { reg: u8, value: u8 },
    Copy { reg_x: u8, reg_y: u8 },
    Call(u16),
    Return,
    SkipIfKey(u8),
    Alu { op: AluOp, reg_x: u8, reg_y: u8 },
    Random(u8),
    SetIndex(u16),
    Halt,
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

pub fn decode(raw: u16) -> Result<Instr, Illegal> {
    let nib = |shift: u32| ((raw >> shift) & 0xF) as u8;
    let imm8 = (raw & 0x00FF) as u8;
    let addr = raw & 0x0FFF;
    let bad = Illegal(raw);

    Ok(match raw {
        0x0000 => Instr::Nop,
        0x1000 => Instr::Clear,
        0x2000..=0x20FF => Instr::DrawColor(imm8),
        0x2100..=0x210F => Instr::DrawColorReg(nib(0)),
        0x2E00..=0x2E0F => Instr::DrawSpriteReg(nib(0)),
        0x2F00..=0x2FFF => Instr::DrawSprite(imm8),
        0x3000..=0x3FFF => Instr::MoveTo(addr),
        0x4000..=0x43FF => match nib(8) {
            0 => Instr::AddX(imm8),
            1 => Instr::SubX(imm8),
            2 => Instr::AddY(imm8),
            3 => Instr::SubY(imm8),
            _ => return Err(bad),
        },
        0x4A00..=0x4A0F => Instr::SetXReg(nib(0)),
        0x4A10..=0x4A1F => Instr::SetYReg(nib(0)),
        0x5000..=0x5FFF => Instr::Jump(addr),
        0x6000..=0x6FFF => Instr::SkipIf {
            cond: Cond::from_u8(nib(8)).ok_or(bad)?,
            reg_x: nib(4),
            reg_y: nib(0),
        },
        0x7000..=0x7FFF => Instr::SetReg {
            reg: nib(8),
            value: imm8,
        },
        0x8000..=0x80FF => Instr::Copy {
            reg_x: nib(0),
            reg_y: nib(4),
        },
        0x9000..=0x9FFF => Instr::Call(addr),
        0xA000 => Instr::Return,
        0xB000..=0xB00F => Instr::SkipIfKey(nib(0)),
        0xC000..=0xCFFF => Instr::Alu {
            op: AluOp::from_u8(nib(8)).ok_or(bad)?,
            reg_x: nib(4),
            reg_y: nib(0),
        },
        0xD000..=0xD00F => Instr::Random(nib(0)),
        0xE000..=0xEFFF => Instr::SetIndex(addr),
        0xFFFF => Instr::Halt,
        _ => return Err(bad),
    })
}
