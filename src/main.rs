
mod token;
mod lexer;
mod ast;
mod parser;
mod bfo_gen;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: hbf <command> <file>");
        eprintln!("Commands:");
        eprintln!("  compile <file.hbf>  - Compile HBF to BFO");
        eprintln!("  build <file.hbf>    - Full pipeline (HBF -> BFO -> BF)");
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "compile" => compile_to_bfo(filename),
        "build" => {
            compile_to_bfo(filename);
            // TODO: Add BF generation
            println!("BF generation not yet implemented");
        },
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn compile_to_bfo(filename: &str) {
    let source = fs::read_to_string(filename).expect("Failed to read file");

    // Lex
    let lexer = lexer::Lexer::new(&source);
    
    // Parse
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();

    // Generate BFO
    let mut generator = bfo_gen::BFOGenerator::new();
    let bfo_code = generator.generate(program);

    // Write BFO file
    let bfo_filename = filename.replace(".hbf", ".bfo");
    fs::write(&bfo_filename, bfo_code).expect("Failed to write BFO file");
    println!("Generated BFO: {}", bfo_filename);
}
