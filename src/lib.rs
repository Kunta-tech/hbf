pub mod hbf_token;
pub mod hbf_lexer;
pub mod hbf_ast;
pub mod hbf_parser;
pub mod bfo_gen;
pub mod bfo_lexer;
pub mod bfo_parser;
pub mod bfo_ast;
pub mod bfo_compiler;
pub mod ir;
pub mod bf_codegen;

use std::fs;

pub fn build_bf(bfo_filename: &str, out_filename: &str) {
    let bfo_source = fs::read_to_string(bfo_filename).expect("Failed to read BFO file");

    let bfo_lexer = bfo_lexer::BFOLexer::new(&bfo_source);
    let mut bfo_parser = bfo_parser::BFOParser::new(bfo_lexer);
    let bfo_program = bfo_parser.parse();

    let mut bfo_compiler = bfo_compiler::BFOCompiler::new();
    let instructions = bfo_compiler.compile(bfo_program);

    let mut bf_codegen = bf_codegen::Codegen::new();
    let bf_code = bf_codegen.generate(&instructions);

    fs::write(out_filename, bf_code).expect("Failed to write BF file");
}

pub fn compile_to_bfo(filename: &str, out_filename: &str) -> String {
    let source = fs::read_to_string(filename).expect("Failed to read file");

    let lexer = hbf_lexer::Lexer::new(&source);
    let mut parser = hbf_parser::Parser::new(lexer);
    let program = parser.parse_program();

    let mut generator = bfo_gen::BFOGenerator::new();
    let bfo_code = generator.generate(program);

    fs::write(out_filename, bfo_code).expect("Failed to write BFO file");
    out_filename.to_string()
}
