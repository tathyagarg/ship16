use clap::Parser;

use crate::{lexer::lex_from_file, resolve::resolve, validate::validate_program};

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

    #[arg(short, long, default_value_t = false)]
    lex: bool,

    #[arg(short, long, default_value_t = false)]
    parse: bool,
}

fn main() {
    let args = Args::parse();
    let tokens = lex_from_file(&args.input).expect("Failed to lex the file");
    if args.lex {
        for token in &tokens {
            dbg!("{:?}", token);
        }
        return;
    }

    let mut parser = parser::Parser::new(&tokens);
    let program = parser.parse_program().expect("Failed to parse the program");
    if args.parse {
        dbg!("{:#?}", program);
        return;
    }

    validate_program(&program).expect("Program validation failed");
    let res = resolve(program).expect("Program resolution failed");
    let bytes = res.encode();

    print!("      ");
    for i in 0..16 {
        if i % 2 == 0 {
            print!("| ");
        }
        print!("{:02X} ", i);
    }
    for (i, byte) in bytes.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04X}: ", i + 0x1000);
        }
        if i % 2 == 0 {
            print!("| ");
        }
        print!("{:02X} ", byte);
    }

    std::fs::write(&args.output, bytes).expect("Failed to write output file");
}
