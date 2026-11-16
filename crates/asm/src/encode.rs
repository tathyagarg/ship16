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

impl ResolvedProgram {
    pub fn encode(mut self) -> Box<[u8]> {
        let mut bytes = Vec::with_capacity(USER_ROM_SIZE);
        let mut label_map: HashMap<String, u16> = self
            .sections
            .iter()
            .map(|s| (s.name.clone(), s.start_address))
            .collect();
        label_map.insert("_SS".to_string(), self.sprite_start as u16);

        for section in &self.sections {
            for instruction in &section.instructions {
                let encoded = encode_instruction(instruction, &label_map);
                bytes.extend_from_slice(&encoded);
            }
        }

        self.sprites.sort_by_key(|sprite| sprite.index);
        let mut last_index = 0;
        for sprite in self.sprites.iter() {
            if sprite.index > last_index {
                let gap = sprite.index - last_index;
                let gap_bytes = gap * 64;
                bytes.extend(std::iter::repeat_n(0, gap_bytes));
            }

            let sprite_bytes = sprite.data;
            bytes.extend_from_slice(&sprite_bytes);

            last_index = sprite.index + 1;
        }

        bytes.into_boxed_slice()
    }
}
