
mod hbf_token;
mod hbf_lexer;
mod hbf_ast;
mod hbf_parser;
mod bfo_gen;
mod bfo_lexer;
mod bfo_parser;
mod bfo_ast;
mod bfo_compiler;
mod ir;
mod bf_codegen;

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
            if !args[2].ends_with(".hbf") {
                eprintln!("Error: 'build' command only supports .hbf files.");
                return;
            }
            let bfo_filename = compile_to_bfo(&args[2]);
            build_bf(&bfo_filename);
        },
        "bf" => {
            if args.len() < 3 {
                eprintln!("Usage: hbf bf <file.bfo>");
                return;
            }
            if !args[2].ends_with(".bfo") {
                eprintln!("Error: 'bf' command only supports .bfo files.");
                return;
            }
            build_bf(&args[2]);
        },
        "test-all" => {
            compile_all_examples();
        },
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn build_bf(bfo_filename: &str) {
    let bfo_source = fs::read_to_string(bfo_filename).expect("Failed to read BFO file");

    // 1. Lex BFO
    let bfo_lexer = bfo_lexer::BFOLexer::new(&bfo_source);

    // 2. Parse BFO
    let mut bfo_parser = bfo_parser::BFOParser::new(bfo_lexer);
    let bfo_program = bfo_parser.parse();

    // 3. Compile BFO to IR
    let mut bfo_compiler = bfo_compiler::BFOCompiler::new();
    let instructions = bfo_compiler.compile(bfo_program);

    // 4. Codegen IR to BF
    let mut bf_codegen = bf_codegen::Codegen::new();
    let bf_code = bf_codegen.generate(&instructions);

    // 5. Write BF file
    let bf_filename = bfo_filename.replace(".bfo", ".bf");
    fs::write(&bf_filename, bf_code).expect("Failed to write BF file");
    println!("Generated BF: {}", bf_filename);
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
                    let bfo_file = compile_to_bfo(path_str);
                    build_bf(&bfo_file);
                }
            }
        }
    }
    println!("Batch compilation complete.");
}

fn compile_to_bfo(filename: &str) -> String {
    let source = fs::read_to_string(filename).expect("Failed to read file");

    // Lex
    let lexer = hbf_lexer::Lexer::new(&source);
    
    // Parse
    let mut parser = hbf_parser::Parser::new(lexer);
    let program = parser.parse_program();

    // Generate BFO
    let mut generator = bfo_gen::BFOGenerator::new();
    let bfo_code = generator.generate(program);

    // Write BFO file
    let bfo_filename = filename.replace(".hbf", ".bfo");
    fs::write(&bfo_filename, bfo_code).expect("Failed to write BFO file");
    println!("Generated BFO: {}", bfo_filename);
    bfo_filename
}
