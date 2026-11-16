use std::collections::HashMap;

use isa::{AluOp, Interrupt};

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    Instruction(Instruction),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub condition: Option<String>,
    pub operands: Vec<Operand>,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(usize),
    Immediate(u16),
    Label(String),
}

impl Operand {
    pub fn expect_register(&self) -> Result<usize, String> {
        match self {
            Operand::Register(reg) => Ok(*reg),
            _ => Err("Expected a register operand".to_string()),
        }
    }

    pub fn expect_immediate(&self) -> Result<u16, String> {
        match self {
            Operand::Immediate(value) => Ok(*value),
            _ => Err("Expected an immediate operand".to_string()),
        }
    }

    pub fn expect_label(&self) -> Result<String, String> {
        match self {
            Operand::Label(label) => Ok(label.clone()),
            _ => Err("Expected a label operand".to_string()),
        }
    }
}

fn decode_directive_to_cond(dir: &str) -> Result<isa::Cond, String> {
    match dir.to_uppercase().as_str() {
        "EQ" => Ok(isa::Cond::Eq),
        "NE" => Ok(isa::Cond::Ne),
        "GT" => Ok(isa::Cond::Gt),
        "LT" => Ok(isa::Cond::Lt),
        "GE" => Ok(isa::Cond::Ge),
        "LE" => Ok(isa::Cond::Le),
        _ => Err(format!("Invalid condition directive: {}", dir)),
    }
}

impl Instruction {
    pub fn to_isa(&self, label_map: &HashMap<String, u16>) -> Result<isa::Instr, String> {
        Ok(match self.opcode.to_uppercase().as_str() {
            "NOP" => isa::Instr::Nop,
            "LD8" => isa::Instr::Load8(self.operands[0].expect_register()?),
            "ST8" => isa::Instr::Store8(self.operands[0].expect_register()?),
            "LD16" => isa::Instr::Load16(self.operands[0].expect_register()?),
            "ST16" => isa::Instr::Store16(self.operands[0].expect_register()?),
            "AI" => {
                if matches!(self.operands[0], Operand::Register(_)) {
                    isa::Instr::AddIndex(self.operands[0].expect_register()?)
                } else if matches!(self.operands[0], Operand::Immediate(_)) {
                    isa::Instr::AddIndexLong(self.operands[0].expect_immediate()?)
                } else {
                    return Err("Invalid operand for AI instruction".to_string());
                }
            }
            "PUSH" => {
                if let Operand::Register(reg) = &self.operands[0] {
                    isa::Instr::Push(*reg)
                } else if let Operand::Immediate(imm) = &self.operands[0] {
                    isa::Instr::PushLong(*imm)
                } else {
                    return Err("Invalid operand for PUSH instruction".to_string());
                }
            }
            "POP" => isa::Instr::Pop(self.operands[0].expect_register()?),
            "FSKP" | "FNSKP" => {
                let flag = match self.operands[0].expect_label()?.as_str() {
                    "Z" => isa::Flag::Zero,
                    "N" => isa::Flag::Negative,
                    "C" => isa::Flag::Carry,
                    "O" => isa::Flag::Overflow,
                    _ => {
                        return Err(format!(
                            "Invalid flag: {}",
                            self.operands[0].expect_label()?
                        ));
                    }
                };

                isa::Instr::SkipIfFlag {
                    flag,
                    set: self.opcode.to_uppercase() == "FSKP",
                }
            }
            "CLS" => isa::Instr::Clear,
            "TC" => isa::Instr::TimerCount(self.operands[0].expect_register()?),
            "TR" => isa::Instr::TimerReload(self.operands[0].expect_register()?),
            "LDTC" => isa::Instr::TimerRead(self.operands[0].expect_register()?),
            "TCFG" => isa::Instr::TimerConfig {
                enable: self.operands[0].expect_immediate()? != 0,
                periodic: self.operands[1].expect_immediate()? != 0,
            },
            "ENINT" => isa::Instr::IntEnable(self.operands[0].expect_immediate()? as u8),
            "DISINT" => isa::Instr::IntDisable(self.operands[0].expect_immediate()? as u8),
            "VEC" => isa::Instr::SetVector(
                Interrupt::from_index(self.operands[0].expect_immediate()? as u8).unwrap(),
            ),
            "EI" => isa::Instr::Ei,
            "DI" => isa::Instr::Di,

            "WAIT" => isa::Instr::Wait,

            "DRW" => isa::Instr::DrawColor(self.operands[0].expect_immediate()? as u8),
            "DRWR" => isa::Instr::DrawColorReg(self.operands[0].expect_register()?),
            "DRWS" => isa::Instr::DrawSprite(self.operands[0].expect_immediate()? as u8),
            "DRWSR" => isa::Instr::DrawSpriteReg(self.operands[0].expect_register()?),

            "MOVC" => {
                let x6 = self.operands[0].expect_immediate()? & 0x3F;
                let y6 = self.operands[1].expect_immediate()? & 0x3F;
                isa::Instr::MoveTo(x6 << 6 | y6)
            }

            "INCX" => isa::Instr::AddX(self.operands[0].expect_immediate()? as u8),
            "INCY" => isa::Instr::AddY(self.operands[0].expect_immediate()? as u8),
            "DECX" => isa::Instr::SubX(self.operands[0].expect_immediate()? as u8),
            "DECY" => isa::Instr::SubY(self.operands[0].expect_immediate()? as u8),
            "SETX" => isa::Instr::SetXReg(self.operands[0].expect_register()?),
            "SETY" => isa::Instr::SetYReg(self.operands[0].expect_register()?),

            "JMP" => {
                isa::Instr::JumpLong(*label_map.get(&self.operands[0].expect_label()?).unwrap())
            }

            "SKP" => {
                let cond = decode_directive_to_cond(
                    self.condition
                        .as_ref()
                        .ok_or("Missing directive for SKP instruction")?,
                )?;

                if matches!(self.operands[1], Operand::Register(_)) {
                    isa::Instr::SkipIf {
                        cond,
                        reg_x: self.operands[0].expect_register()?,
                        reg_y: self.operands[1].expect_register()?,
                    }
                } else if matches!(self.operands[1], Operand::Immediate(_)) {
                    isa::Instr::SkipIfImm {
                        cond,
                        reg_x: self.operands[0].expect_register()?,
                        imm: self.operands[1].expect_immediate()?,
                    }
                } else {
                    return Err("Invalid operand for SKP instruction".to_string());
                }
            }

            "MOV" => {
                if matches!(self.operands[1], Operand::Register(_)) {
                    isa::Instr::Copy {
                        reg_x: self.operands[0].expect_register()?,
                        reg_y: self.operands[1].expect_register()?,
                    }
                } else if matches!(self.operands[1], Operand::Immediate(_)) {
                    isa::Instr::SetRegLong {
                        reg: self.operands[0].expect_register()?,
                        value: self.operands[1].expect_immediate()?,
                    }
                } else {
                    return Err("Invalid operand for MOV instruction".to_string());
                }
            }

            "CALL" => {
                isa::Instr::CallLong(*label_map.get(&self.operands[0].expect_label()?).unwrap())
            }

            "RET" => isa::Instr::Return,
            "RETI" => isa::Instr::Reti,

            "KSKP" => isa::Instr::SkipIfKey(self.operands[0].expect_immediate()? as u8),

            "ADD" | "SUB" | "MUL" | "DIV" | "REM" | "AND" | "OR" | "XOR" | "NOT" | "SHL"
            | "SHR" | "ROL" | "ROR" => {
                let op = AluOp::decode(self.opcode.to_uppercase().as_str())?;

                if op == AluOp::Not {
                    isa::Instr::Alu {
                        op,
                        reg_x: self.operands[0].expect_register()?,
                        reg_y: 0, // Not operation doesn't require a second register
                    }
                } else if matches!(self.operands[1], Operand::Register(_)) {
                    isa::Instr::Alu {
                        op,
                        reg_x: self.operands[0].expect_register()?,
                        reg_y: self.operands[1].expect_register()?,
                    }
                } else if matches!(self.operands[1], Operand::Immediate(_)) {
                    isa::Instr::AluImm {
                        op,
                        reg_x: self.operands[0].expect_register()?,
                        imm: self.operands[1].expect_immediate()?,
                    }
                } else {
                    return Err("Invalid operand for ADD instruction".to_string());
                }
            }

            "RAND" => isa::Instr::Random(self.operands[0].expect_register()?),
            "SETI" => {
                if let Operand::Immediate(imm) = &self.operands[0] {
                    isa::Instr::SetIndexLong(*imm)
                } else {
                    isa::Instr::SetIndexLong(
                        *label_map.get(&self.operands[0].expect_label()?).unwrap(),
                    )
                }
            }

            "HLT" => isa::Instr::Halt,

            _ => panic!("Unsupported opcode: {}", self.opcode),
        })
    }
}
