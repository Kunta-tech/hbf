use super::BFOGenerator;
use crate::hbf_ast::{Expr, Stmt, Type};

impl BFOGenerator {
    pub(super) fn inline_function(&mut self, params: Vec<(Type, String)>, args: Vec<Expr>, body: Vec<Stmt>, return_dest: Option<String>) {
        self.return_stack.push(return_dest);
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
                    Type::Array(inner) => {
                        // Array parameter
                        if let Expr::Variable(array_name) = arg {
                            if let Some(array_info) = self.arrays.get(array_name).cloned() {
                                // Register the parameter name in self.arrays so indexing/length works
                                self.arrays.insert(param_name.clone(), array_info.clone());

                                if !inner.is_virtual() {
                                    // Physical cell array: emit BFO 'ref' for each cell
                                    let (_, length, _, _) = array_info;
                                    for idx in 0..length {
                                        let param_cell = self.get_array_var_name(param_name, idx as i32);
                                        let arg_cell = self.get_array_var_name(array_name, idx as i32);
                                        self.indent();
                                        self.emit_line(&format!("ref {} {}", param_cell, arg_cell));
                                    }
                                }
                            } else {
                                panic!("Array argument '{}' not found", array_name);
                            }
                        } else if inner.is_virtual() {
                            // Virtual array literal: int[] or char[] or bool[]
                            match arg {
                                Expr::StringLiteral(s_val) => {
                                    let char_literals: Vec<Expr> = s_val.chars().map(Expr::CharLiteral).collect();
                                    self.arrays.insert(param_name.clone(), (0, s_val.len(), (**inner).clone(), Some(char_literals)));
                                },
                                Expr::ArrayLiteral(elements) => {
                                    self.arrays.insert(param_name.clone(), (0, elements.len(), (**inner).clone(), Some(elements.clone())));
                                },
                                _ => panic!("Virtual array parameters must be passed by variable name or literal, got {:?}", arg),
                            }
                        } else {
                            panic!("Physical array parameters must be passed by variable name, got {:?}", arg);
                        }
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
        self.return_stack.pop();
    }
}
