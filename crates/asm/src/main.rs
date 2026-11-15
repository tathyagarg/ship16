use clap::Parser;

use crate::{lexer::lex_from_file, validate::validate_program};

mod ast;
mod lexer;
mod parser;
mod validate;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    path: String,
}

fn main() {
    let args = Args::parse();
    let tokens = lex_from_file(&args.path).expect("Failed to lex the file");
    let mut parser = parser::Parser::new(&tokens);
    let program = parser.parse_program().expect("Failed to parse the program");

    validate_program(&program).expect("Program validation failed");

    println!("{:#?}", program);
}
