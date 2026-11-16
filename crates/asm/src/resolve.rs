use isa::{constants::ENTRY_POINT, instruction::instr_size_from_name};

use crate::{ast::*, validate::check_operands};

#[derive(Debug)]
pub struct ResolvedProgram {
    pub sections: Vec<Section>,
    pub sprites: Vec<Sprite>,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub start_address: u16,
    pub instructions: Vec<Instruction>,
}

pub const ALU_OPS: [&str; 13] = [
    "ADD", "SUB", "MUL", "DIV", "REM", "AND", "OR", "XOR", "NOT", "SHL", "SHR", "ROL", "ROR",
];

impl Program {
    pub fn resolve(self) -> ResolvedProgram {
        let mut sections = Vec::new();
        let mut current_section = None;
        let mut current_address = ENTRY_POINT;

        for statement in self.text_section.statements {
            match statement {
                Statement::Label(label) => {
                    if let Some(section) = current_section.take() {
                        sections.push(section);
                    }
                    current_section = Some(Section {
                        name: label.clone(),
                        start_address: current_address,
                        instructions: Vec::new(),
                    });
                }
                Statement::Instruction(instr) => {
                    if let Some(section) = current_section.as_mut() {
                        let mut size = instr_size_from_name(&instr.opcode);
                        if instr.opcode == "SKP"
                            && !check_operands!(instr, [[Register], [Immediate]])
                        {
                            size = 1;
                        } else if instr.opcode == "PUSH" && check_operands!(instr, [[Immediate]]) {
                            size = 2;
                        } else if (ALU_OPS.contains(&instr.opcode.as_str())
                            && check_operands!(instr, [[Register], [Register]]))
                            || (instr.opcode == "MOV"
                                && check_operands!(instr, [[Register], [Register]]))
                            || (instr.opcode == "AI" && check_operands!(instr, [[Register]]))
                        {
                            size = 1;
                        }

                        println!("Instruction: {}, Size: {}", instr.opcode, size);

                        current_address += 2 * size;
                        section.instructions.push(instr.clone());
                    }
                }
            }
        }

        if let Some(section) = current_section {
            sections.push(section);
        }

        ResolvedProgram {
            sections,
            sprites: self.sprite_section.sprites,
        }
    }
}

fn move_start_to_beginning(program: &mut Program) {
    if let Some(start_index) = program.text_section.statements.iter().position(|stmt| {
        if let Statement::Label(label) = stmt {
            label.eq_ignore_ascii_case("START")
        } else {
            false
        }
    }) {
        let mut start_statements = program.text_section.statements.split_off(start_index);
        start_statements.append(&mut program.text_section.statements);
        program.text_section.statements = start_statements;
    } else {
        program
            .text_section
            .statements
            .insert(0, Statement::Label("START".to_string()));
    }
}

pub fn resolve(mut program: Program) -> Result<ResolvedProgram, String> {
    move_start_to_beginning(&mut program);
    Ok(program.resolve())
}
