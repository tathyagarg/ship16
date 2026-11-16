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
        let mut statements = Vec::new();

        while let Some(token) = self.current() {
            match token {
                Token::Identifier(_) => {
                    let statement = self.parse_statement()?;
                    statements.push(statement);
                }
                Token::Newline | Token::Comment(_) => {
                    self.advance(); // Skip newlines
                }
                _ => {
                    return Err(format!("Unexpected token: {:?}", token));
                }
            }
        }

        Ok(Program { statements })
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
}
