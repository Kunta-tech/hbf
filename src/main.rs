
mod token;
mod lexer;
mod ast;
mod parser;
mod ir;
mod compiler;
mod codegen;

use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: hbf build <file.hbf>");
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    if command != "build" {
        eprintln!("Unknown command: {}", command);
        return;
    }

    let source = fs::read_to_string(filename).expect("Failed to read file");

    // 1. Lex
    let mut lexer = lexer::Lexer::new(&source);
    
    // 2. Parse
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();

    // 3. Compile (HBF -> BFO)
    let mut compiler = compiler::Compiler::new();
    let bfo_ops = compiler.compile(program);

    // 4. Save BFO (Debug)
    let bfo_filename = filename.replace(".hbf", ".bfo");
    let bfo_debug = format!("{:?}", bfo_ops);
    fs::write(&bfo_filename, bfo_debug).expect("Failed to write BFO file");
    println!("Generated BFO: {}", bfo_filename);

    // 5. Codegen (BFO -> BF)
    let mut codegen = codegen::Codegen::new();
    let bf_code = codegen.generate(&bfo_ops);

    let bf_filename = filename.replace(".hbf", ".bf");
    fs::write(&bf_filename, bf_code).expect("Failed to write BF file");
    println!("Generated BF: {}", bf_filename);
}
