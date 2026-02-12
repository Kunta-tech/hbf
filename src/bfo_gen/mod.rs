
use crate::hbf_ast::{Expr, Stmt, Program, Type};
use std::collections::HashMap;

mod scope;
mod emit;
mod expr_fold;
mod stmt_gen;
mod inline;

pub struct BFOGenerator {
    pub(super) output: String,
    pub(super) functions: HashMap<String, Stmt>, // Store function definitions
    pub(super) arrays: HashMap<String, (usize, usize, Type, Option<Vec<Expr>>)>, // name -> (base_addr, length, element_type, literals)
    pub(super) variables: Vec<HashMap<String, Expr>>, // Scoped virtual variables (int, char)
    pub(super) indent_level: usize,
    pub(super) forn_counter: usize,
    pub(super) native_loop_depth: usize,
    pub(super) return_stack: Vec<Option<String>>,
}

impl BFOGenerator {
    pub fn new() -> Self {
        BFOGenerator {
            output: String::new(),
            functions: HashMap::new(),
            arrays: HashMap::new(),
            variables: vec![HashMap::new()],
            indent_level: 0,
            forn_counter: 0,
            native_loop_depth: 0,
            return_stack: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: Program) -> String {
        for stmt in program.statements {
            match &stmt {
                Stmt::FuncDecl { name, .. } => {
                    self.emit(&format!("; defined function {}\n", name));
                    self.functions.insert(name.clone(), stmt.clone());
                }
                _ => {
                    self.gen_stmt(stmt, true);
                }
            }
        }
        self.output.clone()
    }
}
