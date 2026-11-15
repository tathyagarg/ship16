use crate::ast::*;

const VALID_OPCODES: [&str; 55] = [
    "NOP", "LD8", "ST8", "LD16", "ST16", "AI", "PUSH", "POP", "FSKP", "FNSKP", "CLS", "TC", "TR",
    "LDTC", "TCFG", "ENINT", "DISINT", "VEC", "EI", "DI", "WAIT", "DRW", "DRWR", "DRWS", "DRWSR",
    "MOVC", "INCX", "INCY", "DECX", "DECY", "SETX", "SETY", "JMP", "SKP", "MOV", "CALL", "RET",
    "RETI", "KSKP", "ADD", "SUB", "MUL", "DIV", "REM", "AND", "OR", "XOR", "NOT", "SHL", "SHR",
    "ROL", "ROR", "RAND", "SETI", "HLT",
];

#[derive(Debug, PartialEq, Eq)]
pub enum OperandType {
    Register,
    Immediate,
    Label,
}

macro_rules! check_operands {
    ($instruction:expr, [$([$($expected:ident)|+]),* $(,)?]) => {{
        let expected: Vec<Vec<OperandType>> = vec![
            $(
                vec![$(OperandType::$expected),+]
            ),*
        ];

        let actual: Vec<OperandType> = $instruction
            .operands
            .iter()
            .map(|operand| match operand {
                Operand::Register(_) => OperandType::Register,
                Operand::Immediate(_) => OperandType::Immediate,
                Operand::Label(_) => OperandType::Label,
            })
            .collect();

        actual.len() == expected.len()
            && actual.iter()
                .zip(expected.iter())
                .all(|(actual, allowed)| allowed.contains(actual))
    }};
}

macro_rules! validate_instruction {
    ($instruction:expr, {
        $(
            $opcode:literal => [$([$($expected:ident)|+]),* $(,)?]
        ),* $(,)?
    }) => {{
        match $instruction.opcode.as_str() {
            $(
                $opcode => check_operands!(
                    $instruction,
                    [$([$($expected)|+]),*]
                ),
            )*
            _ => false,
        }
    }};
}

fn is_valid_instruction(instr: &Instruction) -> Result<(), String> {
    let opcode = instr.opcode.to_uppercase();
    if !VALID_OPCODES.contains(&opcode.as_str()) {
        return Err(format!("Invalid opcode: {}", instr.opcode));
    }

    let result = validate_instruction!(instr, {
        "NOP" => [],
        "LD8" => [[Register]],
        "ST8" => [[Register]],
        "LD16" => [[Register]],
        "ST16" => [[Register]],
        "AI" => [[Register]],
        "PUSH" => [[Register]],
        "POP" => [[Register]],
        "FSKP" => [[Label]],
        "FNSKP" => [[Label]],
        "CLS" => [],
        "TC" => [[Register]],
        "TR" => [[Register]],
        "LDTC" => [[Register]],
        "TCFG" => [[Immediate], [Immediate]],
        "ENINT" => [[Immediate]],
        "DISINT" => [[Immediate]],
        "VEC" => [[Immediate]],
        "EI" => [],
        "DI" => [],
        "WAIT" => [],
        "DRW" => [[Immediate]],
        "DRWR" => [[Register]],
        "DRWS" => [[Immediate]],
        "DRWSR" => [[Register]],
        "MOVC" => [[Immediate], [Immediate]],
        "INCX" => [[Immediate]],
        "INCY" => [[Immediate]],
        "DECX" => [[Immediate]],
        "DECY" => [[Immediate]],
        "SETX" => [[Register]],
        "SETY" => [[Register]],
        "JMP" => [[Label]],
        "SKP" => [[Register], [Register | Immediate]],
        "MOV" => [[Register], [Register | Immediate]],
        "CALL" => [[Label]],
        "RET" => [],
        "RETI" => [],
        "KSKP" => [[Immediate]],
        "ADD" => [[Register], [Register | Immediate]],
        "SUB" => [[Register], [Register | Immediate]],
        "MUL" => [[Register], [Register | Immediate]],
        "DIV" => [[Register], [Register | Immediate]],
        "REM" => [[Register], [Register | Immediate]],
        "AND" => [[Register], [Register | Immediate]],
        "OR" => [[Register], [Register | Immediate]],
        "XOR" => [[Register], [Register | Immediate]],
        "NOT" => [[Register]],
        "SHL" => [[Register], [Immediate]],
        "SHR" => [[Register], [Immediate]],
        "ROL" => [[Register], [Immediate]],
        "ROR" => [[Register], [Immediate]],
        "RAND" => [[Register]],
        "SETI" => [[Label]],
        "HLT" => [],
    });

    if !result {
        return Err(format!(
            "Invalid operands for instruction: {} {:?}",
            instr.opcode, instr.operands
        ));
    }

    if instr.directive.is_some() && instr.opcode != "SKP" {
        return Err(format!(
            "Directives are only allowed for SKP instruction, found in: {}",
            instr.opcode
        ));
    }

    if matches!(instr.opcode.as_str(), "FSKP" | "FNSKP")
        && !matches!(instr.operands.get(0), Some(Operand::Label(label)) if label == "Z" || label == "N" || label == "C" || label == "O")
    {
        return Err(format!(
            "FSKP and FNSKP instructions must have a flag operand of Z, N, C, or O, found: {:?}",
            instr.operands.get(0)
        ));
    }

    Ok(())
}

pub fn validate_program(program: &Program) -> Result<(), String> {
    let mut labels = std::collections::HashSet::new();

    for statement in &program.statements {
        match statement {
            Statement::Label(label) => {
                if !labels.insert(label.clone()) {
                    return Err(format!("Duplicate label found: {}", label));
                }
            }
            Statement::Instruction(instruction) => {
                is_valid_instruction(instruction)?;
            }
        }
    }

    Ok(())
}
