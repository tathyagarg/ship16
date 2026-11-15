use clap::Parser;

use crate::lexer::lex_from_file;

mod lexer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    path: String,
}

fn main() {
    let args = Args::parse();
    let tokens = lex_from_file(&args.path).expect("Failed to lex the file");
    println!("Got {} tokens from file: {}", tokens.len(), args.path);
    for token in tokens.iter() {
        println!("{:?}", token);
    }
}
