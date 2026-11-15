use isa::instruction::instr_size_from_name;

use crate::{ast::*, validate::check_operands};

#[derive(Debug)]
pub struct ResolvedProgram {
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub start_address: u16,
    pub instructions: Vec<Instruction>,
}

impl Program {
    pub fn resolve(self) -> ResolvedProgram {
        let mut sections = Vec::new();
        let mut current_section = None;
        let mut current_address = 0x900u16;

        for (i, statement) in self.statements.iter().enumerate() {
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
                            && check_operands!(instr, [[Register], [Immediate]])
                        {
                            size = 1;
                        }

                        current_address += 2 * size;
                        section.instructions.push(instr.clone());
                    }
                }
            }
        }

        if let Some(section) = current_section {
            sections.push(section);
        }

        ResolvedProgram { sections }
    }
}

fn move_start_to_beginning(program: &mut Program) {
    if let Some(start_index) = program.statements.iter().position(|stmt| {
        if let Statement::Label(label) = stmt {
            label.to_ascii_uppercase() == "START"
        } else {
            false
        }
    }) {
        let mut start_statements = program.statements.split_off(start_index);
        start_statements.append(&mut program.statements);
        program.statements = start_statements;
    } else {
        program
            .statements
            .insert(0, Statement::Label("START".to_string()));
    }
}

pub fn resolve(mut program: Program) -> Result<ResolvedProgram, String> {
    move_start_to_beginning(&mut program);
    Ok(program.resolve())
}
