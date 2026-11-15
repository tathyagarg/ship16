use std::collections::HashMap;

use crate::{ast::Instruction, resolve::ResolvedProgram};

fn encode_instruction(instr: &Instruction, label_map: &HashMap<String, u16>) -> Box<[u8]> {
    let isa_instr = instr
        .to_isa(label_map)
        .unwrap_or_else(|_| panic!("Failed to encode instruction: {:?}", instr));

    let (w1, w2) = isa_instr.encode();
    let result = if let Some(w2) = w2 {
        vec![
            (w1 & 0xFF) as u8,
            (w1 >> 8) as u8,
            (w2 & 0xFF) as u8,
            (w2 >> 8) as u8,
        ]
    } else {
        vec![(w1 & 0xFF) as u8, (w1 >> 8) as u8]
    };

    result.into_boxed_slice()
}

pub fn encode(program: &ResolvedProgram) -> Box<[u8]> {
    let mut bytes = Vec::new();
    let label_map: HashMap<String, u16> = program
        .sections
        .iter()
        .map(|s| (s.name.clone(), s.start_address))
        .collect();

    for section in &program.sections {
        for instruction in &section.instructions {
            let encoded = encode_instruction(instruction, &label_map);
            bytes.extend_from_slice(&encoded);
        }
    }

    bytes.into_boxed_slice()
}
