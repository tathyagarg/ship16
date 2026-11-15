#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Label(String),
    Instruction(Instruction),
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: String,
    pub directive: Option<String>,
    pub operands: Vec<Operand>,
}

#[derive(Debug)]
pub enum Operand {
    Register(u8),
    Immediate(u16),
    Label(String),
}
