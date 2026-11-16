use crate::{ast::*, lexer::Token};

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut text_section = None;
        let mut sprite_section = None;

        while let Some(token) = self.current() {
            match token {
                Token::Newline | Token::Comment(_) => {
                    self.advance();
                }
                Token::Directive(directive) if directive == "section" => {
                    self.advance();
                    if let Some(Token::Identifier(section_name)) = self.current() {
                        match section_name.as_str() {
                            "text" => {
                                self.advance();
                                let text = self.parse_text()?;
                                text_section = Some(text);
                            }
                            "sprites" => {
                                self.advance();
                                let sprites = self.parse_sprite()?;
                                sprite_section = Some(sprites);
                            }
                            _ => {
                                return Err(format!("Unknown section name: {}", section_name));
                            }
                        }
                    } else {
                        return Err("Expected section name after .section directive".to_string());
                    }
                }
                _ => {
                    return Err(format!("Unexpected token: {:?}", token));
                }
            }
        }

        Ok(Program {
            text_section: text_section.unwrap_or(TextSection {
                statements: Vec::new(),
            }),
            sprite_section: sprite_section.unwrap_or(SpriteSection {
                sprites: Vec::new(),
            }),
        })
    }

    pub fn parse_text(&mut self) -> Result<TextSection, String> {
        let mut statements = Vec::new();

        while let Some(token) = self.current() {
            match token {
                Token::Identifier(_) => {
                    let statement = self.parse_statement()?;
                    statements.push(statement);
                }
                Token::Newline | Token::Comment(_) => {
                    self.advance();
                }
                Token::Directive(directive) if directive == "section" => {
                    break;
                }
                _ => {
                    return Err(format!("Unexpected token: {:?}", token));
                }
            }
        }

        Ok(TextSection { statements })
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        let curr = self.current();

        if let Some(Token::Identifier(_)) = curr {
            let next = self.peek();

            if let Some(Token::Colon) = next {
                let label = self.parse_label()?;
                Ok(Statement::Label(label))
            } else {
                let instruction = self.parse_instruction()?;
                Ok(Statement::Instruction(instruction))
            }
        } else {
            Err("Expected a statement".to_string())
        }
    }

    pub fn parse_instruction(&mut self) -> Result<Instruction, String> {
        if let Some(Token::Identifier(opcode)) = self.current() {
            let opcode_name = opcode.clone();
            self.advance(); // Advance past the opcode

            let condition = if let Some(Token::Condition(cond)) = self.current() {
                let condition_name = cond.clone();
                self.advance(); // Advance past the directive
                Some(condition_name)
            } else {
                None
            };

            let mut operands = Vec::new();
            if let Some(Token::Identifier(_))
            | Some(Token::Number(_))
            | Some(Token::HexNumber(_))
            | Some(Token::BinaryNumber(_)) = self.current()
            {
                let operand = self.parse_operand()?;
                operands.push(operand);

                while let Some(Token::Comma) = self.current() {
                    self.advance(); // Advance past the comma
                    let operand = self.parse_operand()?;
                    operands.push(operand);
                }
            }

            Ok(Instruction {
                opcode: opcode_name,
                condition,
                operands,
            })
        } else {
            Err("Expected an instruction".to_string())
        }
    }

    pub fn parse_label(&mut self) -> Result<String, String> {
        if let Some(Token::Identifier(label)) = self.current() {
            if let Some(Token::Colon) = self.peek() {
                let label_name = label.clone();
                self.advance(); // Advance past the identifier
                self.advance(); // Advance past the colon
                Ok(label_name)
            } else {
                Err("Expected a colon after label".to_string())
            }
        } else {
            Err("Expected a label".to_string())
        }
    }

    pub fn parse_operand(&mut self) -> Result<Operand, String> {
        if let Some(curr) = self.current() {
            match curr.clone() {
                Token::Identifier(reg) if reg.starts_with("r") => {
                    self.advance();
                    let reg_num = usize::from_str_radix(&reg[1..], 16)
                        .map_err(|e| format!("Invalid register number: {} {}", e, reg))?;
                    if reg_num > 15 {
                        return Err("Register number out of range (0-15)".to_string());
                    }
                    Ok(Operand::Register(reg_num))
                }
                Token::Number(value) | Token::HexNumber(value) | Token::BinaryNumber(value) => {
                    self.advance();
                    Ok(Operand::Immediate(value))
                }
                Token::Identifier(label) => {
                    self.advance();
                    Ok(Operand::Label(label.clone()))
                }
                _ => Err("Expected an operand".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing operand".to_string())
        }
    }

    pub fn parse_sprite(&mut self) -> Result<SpriteSection, String> {
        let mut sprites = Vec::new();

        while let Some(token) = self.current() {
            match token {
                Token::Directive(directive) if directive == "sprite" => {
                    self.advance();
                    if let Some(Token::Number(index)) = self.current() {
                        let sprite_index = *index as usize;
                        self.advance();
                        let sprite_data = self.parse_sprite_data()?;
                        sprites.push(Sprite {
                            index: sprite_index,
                            data: sprite_data,
                        });
                    }
                }
                Token::Newline | Token::Comment(_) => {
                    self.advance();
                }
                _ => {
                    return Err(format!("Unexpected token: {:?}", token));
                }
            }
        }

        Ok(SpriteSection { sprites })
    }

    pub fn parse_sprite_data(&mut self) -> Result<[u8; 64], String> {
        let mut data = [0u8; 64];
        let mut count = 0;

        while let Some(token) = self.current() {
            match token {
                Token::HexNumber(value) => {
                    if count >= 64 {
                        return Err("Sprite data exceeds 64 bytes".to_string());
                    }
                    data[count] = *value as u8;
                    count += 1;
                    self.advance();
                    if let Some(Token::Comma) = self.current() {
                        self.advance();
                    }
                }
                Token::Newline | Token::Comment(_) => {
                    self.advance();
                }
                _ => {
                    break;
                }
            }
        }

        if count != 64 {
            return Err(format!("Expected 64 bytes of sprite data, found {}", count));
        }

        Ok(data)
    }
}
