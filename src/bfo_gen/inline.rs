use super::BFOGenerator;
use crate::hbf_ast::{Expr, Stmt, Type};

impl BFOGenerator {
    pub(super) fn inline_function(&mut self, params: Vec<(Type, String)>, args: Vec<Expr>, body: Vec<Stmt>) {
        // 1. Evaluate arguments in CURRENT scope
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.fold_expr(arg));
        }

        // 2. Push NEW scope for the function body (virtual scope)
        self.push_scope();
        
        // 3. Start BFO block (runtime scope)
        self.indent();
        self.emit_line("{");
        self.indent_level += 1;

        // 4. Initialize parameters
        for (i, (param_type, param_name)) in params.iter().enumerate() {
            if let Some(arg) = evaluated_args.get(i) {
                match param_type {
                    Type::Cell => {
                        // Pass-by-value: Create new local cell initialized with argument value
                        // This 'new' will happen inside the BFO block, shadowing any outer 'param_name'
                        self.materialize_to_cell(param_name, arg.clone(), true);
                    },
                    _ => {
                        // Virtual variables (int, char): simulate via aliasing/folding
                        self.declare_variable(param_name, arg.clone());
                    }
                }
            }
        }

        // 5. Generate code for the body
        for stmt in body {
            self.gen_stmt(stmt, false);
        }

        // 6. End BFO block
        self.indent_level -= 1;
        self.indent();
        self.emit_line("}");

        // 7. Pop scope
        self.pop_scope();
    }
}
