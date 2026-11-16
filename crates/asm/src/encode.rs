use std::collections::HashMap;

use isa::constants::*;

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

pub fn encode(program: &ResolvedProgram) -> [u8; USER_ROM_SIZE] {
    let mut bytes = [0u8; USER_ROM_SIZE];
    let label_map: HashMap<String, u16> = program
        .sections
        .iter()
        .map(|s| (s.name.clone(), s.start_address))
        .collect();

    let mut curr_pos = 0usize;

    for section in &program.sections {
        for instruction in &section.instructions {
            let encoded = encode_instruction(instruction, &label_map);
            for byte in encoded.iter() {
                if curr_pos < USER_ROM_SIZE {
                    bytes[curr_pos] = *byte;
                    curr_pos += 1;
                } else {
                    panic!("ROM size exceeded while encoding instructions");
                }
            }
        }
    }

    curr_pos = ROM_CUSTOM_SPRITE_BASE - ENTRY_POINT as usize;
    for sprite in &program.sprites {
        let sprite_bytes = sprite.data;
        for byte in sprite_bytes.iter() {
            if curr_pos < USER_ROM_SIZE {
                bytes[curr_pos] = *byte;
                curr_pos += 1;
            } else {
                panic!("ROM size exceeded while encoding sprites");
            }
        }
    }

    bytes
}
