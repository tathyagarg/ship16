use isa::instruction::{AluOp, AluOut, Cond};

pub trait Evaluate {
    type Output;

    fn eval(&self, a: u16, b: u16) -> Self::Output;
}

impl Evaluate for AluOp {
    type Output = AluOut;

    fn eval(&self, a: u16, b: u16) -> Self::Output {
        let out = |result, carry, overflow| AluOut {
            result,
            carry,
            overflow,
        };

        match self {
            AluOp::Add => {
                let (result, carry) = a.overflowing_add(b);
                let overflow = ((a ^ result) & (b ^ result) & 0x8000) != 0;
                out(result, carry, overflow)
            }
            AluOp::Sub => {
                let (result, carry) = a.overflowing_sub(b);
                let overflow = ((a ^ b) & (a ^ result) & 0x8000) != 0;
                out(result, carry, overflow)
            }
            AluOp::Mul => {
                let result = a.wrapping_mul(b);
                let carry = (a as u32 * b as u32) > 0xFFFF;
                let overflow = false;
                out(result, carry, overflow)
            }
            AluOp::Div => match a.checked_div(b) {
                Some(result) => out(result, false, false),
                None => out(0, true, true),
            },
            AluOp::Rem => match a.checked_rem(b) {
                Some(result) => out(result, false, false),
                None => out(0, true, true),
            },
            AluOp::And => out(a & b, false, false),
            AluOp::Or => out(a | b, false, false),
            AluOp::Xor => out(a ^ b, false, false),
            AluOp::Not => out(!a, false, false),
            AluOp::Shl => match b {
                0 => out(a, false, false),
                1..=16 => {
                    let wide = (a as u32) << b;
                    out(wide as u16, (wide >> 16) & 1 != 0, false)
                }
                _ => out(0, true, true),
            },
            AluOp::Shr => match b {
                0 => out(a, false, false),
                1..=16 => {
                    let wide = (a as u32) >> b;
                    out(wide as u16, (a >> (b - 1)) & 1 != 0, false)
                }
                _ => out(0, true, true),
            },
            AluOp::Rol => {
                let r = a.rotate_left(b as u32);
                out(r, b % 16 != 0 && r & 1 != 0, false)
            }
            AluOp::Ror => {
                let r = a.rotate_right(b as u32);
                out(r, b % 16 != 0 && r & 0x8000 != 0, false)
            }
        }
    }
}

impl Evaluate for Cond {
    type Output = bool;

    fn eval(&self, a: u16, b: u16) -> Self::Output {
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
