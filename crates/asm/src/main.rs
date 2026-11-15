use clap::Parser;

use crate::{encode::encode, lexer::lex_from_file, resolve::resolve, validate::validate_program};

mod ast;
mod encode;
mod lexer;
mod parser;
mod resolve;
mod validate;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    input: String,

    #[arg(short, long, value_name = "FILE", default_value_t = String::from("output.bin"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    let tokens = lex_from_file(&args.input).expect("Failed to lex the file");
    let mut parser = parser::Parser::new(&tokens);
    let program = parser.parse_program().expect("Failed to parse the program");

    validate_program(&program).expect("Program validation failed");
    let res = resolve(program).expect("Program resolution failed");
    dbg!("Resolved program: {:?}", &res);

    let bytes = encode(&res);

    for (i, byte) in bytes.iter().enumerate() {
        if i % 2 == 0 {
            print!("\n{:04X}: ", i);
        }
        print!("{:02X} ", byte);
    }

    std::fs::write(&args.output, &bytes).expect("Failed to write output file");
}
