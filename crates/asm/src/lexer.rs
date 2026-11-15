use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    Directive(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u16>().unwrap())]
    Number(u16),

    #[regex(r"0x[0-9a-fA-F]+", |lex| u16::from_str_radix(&lex.slice()[2..], 16).unwrap())]
    HexNumber(u16),

    #[regex(r"0b[01]+", |lex| u16::from_str_radix(&lex.slice()[2..], 2).unwrap())]
    BinaryNumber(u16),

    #[token(r":")]
    Colon,

    #[token(r",")]
    Comma,

    #[regex(r";[^\n]*", |lex| lex.slice().to_string(), allow_greedy = true)]
    Comment(String),

    #[regex(r"\n+")]
    Newline,
}

pub fn lex(input: &str) -> Box<[Token]> {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next() {
        if let Ok(token) = token {
            tokens.push(token);
        }
    }

    tokens.into_boxed_slice()
}

pub fn lex_from_file(path: &str) -> Result<Box<[Token]>, Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(path)?;
    Ok(lex(&input))
}
