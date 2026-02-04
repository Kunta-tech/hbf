
mod token;
mod lexer;
mod ast;
mod parser;
mod bfo_gen;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hbf <command> [file]");
        eprintln!("Commands:");
        eprintln!("  compile <file.hbf>  - Compile HBF to BFO");
        eprintln!("  build <file.hbf>    - Full pipeline (HBF -> BFO -> BF)");
        eprintln!("  test-all            - Compile all .hbf files in examples/");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "compile" => {
            if args.len() < 3 {
                eprintln!("Usage: hbf compile <file.hbf>");
                return;
            }
            compile_to_bfo(&args[2]);
        },
        "build" => {
            if args.len() < 3 {
                eprintln!("Usage: hbf build <file.hbf>");
                return;
            }
            compile_to_bfo(&args[2]);
            println!("BF generation not yet implemented");
        },
        "test-all" => {
            compile_all_examples();
        },
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn compile_all_examples() {
    let entries = fs::read_dir("examples").expect("Could not read examples directory");
    println!("Compiling all benchmarks/examples...");
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "hbf") {
                if let Some(path_str) = path.to_str() {
                    println!("Testing {}", path_str);
                    compile_to_bfo(path_str);
                }
            }
        }
    }
    println!("Batch compilation complete.");
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
