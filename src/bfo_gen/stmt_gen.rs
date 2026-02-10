use super::BFOGenerator;
use crate::hbf_ast::{Expr, Stmt, Type};
use crate::hbf_token::Token;

impl BFOGenerator {
    pub(super) fn gen_stmt(&mut self, stmt: Stmt, is_top_level: bool) {
        match stmt {
            Stmt::VarDecl { var_type, name, value } => {
                let folded = self.fold_expr(value);
    
                match &var_type {
                    Type::Cell => self.materialize_to_cell(&name, folded, true),
                    Type::Array(inner) => {
                        if !inner.is_virtual() {
                            // cell[] is physical: contiguous named cells
                            if let Expr::ArrayLiteral(elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), None));
                                for (i, el) in elements.iter().enumerate() {
                                    let cell_name = self.get_array_var_name(&name, i as i32);
                                    self.materialize_to_cell(&cell_name, el.clone(), true);
                                }
                            } else {
                                // Default array init? Usually arrays are sized.
                                self.arrays.insert(name, (0, 0, (**inner).clone(), None));
                            }
                        } else {
                            // int[] or char[] or bool[] is virtual: store in memory
                            if let Expr::StringLiteral(ref s_val) = folded {
                                let char_literals: Vec<Expr> = s_val.chars().map(Expr::CharLiteral).collect();
                                self.arrays.insert(name.clone(), (0, s_val.len(), (**inner).clone(), Some(char_literals)));
                            } else if let Expr::ArrayLiteral(ref elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), Some(elements.clone())));
                            }
                        }
                    },
                    _ => {
                        // int or char is virtual: store in memory (current scope)
                        self.declare_variable(&name, folded);
                    }
                }
            },
            Stmt::IndexedAssign { name, index, value } => {
                let folded_val = self.fold_expr(value);
                let folded_index = self.fold_expr(index);
                if let Expr::Number(i) = folded_index {
                    if let Some((_, _, elem_type, literals)) = self.arrays.get_mut(&name) {
                        if !elem_type.is_virtual() {
                            // physical cell[] update
                            let cell_name = self.get_array_var_name(&name, i);
                            self.materialize_to_cell(&cell_name, folded_val, false);
                        } else {
                            // virtual array update (silent)
                            if self.native_loop_depth > 0 {
                                eprintln!("WARNING: Modifying virtual array '{}' inside a native loop. This value will not update in the generated BFO.", name);
                            }
                            if let Some(lits) = literals {
                                if (i as usize) < lits.len() {
                                    lits[i as usize] = folded_val;
                                }
                            }
                        }
                    }
                }
            },
            Stmt::Assign { name, value } => {
                let folded = self.fold_expr(value);
                // Determine if 'name' is physical or virtual
                if self.get_variable(&name).is_some() {
                    if self.native_loop_depth > 0 {
                        eprintln!("WARNING: Modifying virtual variable '{}' inside a native loop. This value will not update in the generated BFO.", name);
                    }
                    self.set_variable(&name, folded);
                } else {
                    self.materialize_to_cell(&name, folded, false);
                }
            },
            Stmt::FuncDecl { name, params: _, return_type: _, body: _ } => {
                if is_top_level {
                     // We already collected functions in the first pass.
                     // In simulation-only mode, we do NOT emit the function definition to BFO.
                     // Debug/comment line:
                     self.emit_line(&format!("; defined function {}", name));
                }
            },
            Stmt::Putc(expr) => {
                let folded = self.fold_expr(expr);
    
                match folded {
                    Expr::Variable(name) => {
                        // If it's a virtual array, print sequentially
                        let array_data = if let Some((_, _, elem_type, literals)) = self.arrays.get(&name) {
                            if *elem_type == Type::Char {
                                literals.clone().map(|lits| (name.clone(), lits))
                            } else { None }
                        } else { None };

                        if let Some((name, lits)) = array_data {
                            for (_i, lit) in lits.iter().enumerate() {
                                self.materialize_to_cell(&name, lit.clone(), false);
                                self.indent();
                                self.emit_line(&format!("print {}", name));
                            }
                        } else {
                            // Single physical variable or unknown
                            self.indent();
                            self.emit_line(&format!("print {}", name));
                        }
                    },
                    Expr::ArrayAccess { array, index } => {
                        // If it's a physical cell array access, we can print the cell name directly
                        if let Expr::Number(i) = index.as_ref() {
                            if let Expr::Variable(array_name) = array.as_ref() {
                                if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                                    if !elem_type.is_virtual() {
                                        let cell_name = self.get_array_var_name(array_name, *i);
                                        self.indent();
                                        self.emit_line(&format!("print {}", cell_name));
                                        return;
                                    }
                                }
                            }
                        }
            
                        // Otherwise fall back to materialization
                        self.materialize_to_cell("__hbf_tmp", Expr::ArrayAccess { array, index }, false);
                        self.indent();
                        self.emit_line("print __hbf_tmp");
                    },
                    Expr::Number(n) => {
                        self.indent();
                        self.emit_line(&format!("print {}", n));
                    },
                    Expr::CharLiteral(c) => {
                        self.indent();
                        self.emit_line(&format!("print '{}'", c.escape_default()));
                    },
                    Expr::StringLiteral(s) => {
                        for ch in s.chars() {
                            self.indent();
                            self.emit_line(&format!("print '{}'", ch.escape_default()));
                        }
                    },
                    Expr::ArrayLiteral(elements) => {
                        for el in elements {
                            self.indent();
                            self.emit("print ");
                            self.gen_expr_simple(el.clone());
                            self.emit_line("");
                        }
                    },
                    _ => {
                        // Complex expression materialization
                        // Use local 'tmp' cell for BFO printing
                        self.materialize_to_cell("__hbf_tmp", folded, false);
                        self.indent();
                        self.emit_line("print __hbf_tmp");
                        self.emit_line("free __hbf_tmp");
                    }
                }
            },
            Stmt::For { init, condition, update, body } => {
                // for(;;) is strictly for unfolding (simulation)
                self.push_scope();
                if let Some(i) = init {
                    self.gen_stmt(*i.clone(), false);
                }
    
                let mut iterations = 0;
                while iterations < 10000 {

                    let cond_val = if let Some(cond) = &condition {
                        let folded = self.fold_expr(cond.clone());
                        
                        match folded {
                            Expr::BoolLiteral(b) => b,
                            Expr::Number(n) => n != 0,
                            _ => {
                                panic!("For loop condition must be a compile-time constant for unfolding, got {:?}", folded);
                            }
                        }
                    } else {
                        true
                    };

                    if !cond_val { break; }
        
                    for s in &body {
                        self.gen_stmt(s.clone(), false);
                    }
        
                    if let Some(ref u) = update {
                        self.gen_stmt(*u.clone(), false);
                    }
                    iterations += 1;
                }
    
                if iterations >= 10000 {
                    panic!("Loop unrolling exceeded limit (possible infinite loop or too large)");
                }
                self.pop_scope();
            },
            Stmt::Forn { count, body } => {
                let folded_count = self.fold_expr(count.clone());
    
                // Determine the loop counter name
                let mut is_var = false;
                let name = match &count {
                    Expr::Variable(n) => {
                        if let Some(_val) = self.get_variable(n) {
                            // Generate an anonymous counter
                            let for_name = format!("__hbf_forn_{}", self.forn_counter);
                            self.forn_counter += 1;
                            is_var = true;
                            for_name
                        } else {
                            n.clone()
                        }
                    },
                    _ => {
                        // Generate an anonymous counter
                        let n = format!("__hbf_forn_{}", self.forn_counter);
                        self.forn_counter += 1;
                        is_var = true;
                        n
                    }
                };

                // Generate native countdown loop: set n value; while n { body; sub n 1 }
                if is_var { 
                    self.materialize_to_cell(&name, folded_count, true); 
                } else {
                    self.materialize_to_cell(&name, folded_count, false); 
                }
    
                self.indent();
                self.emit_line(&format!("while {} {{", name));
    
                self.indent_level += 1;
                self.native_loop_depth += 1;

                for s in body {
                    self.gen_stmt(s, false);
                }
                self.native_loop_depth -= 1;
    
                // Decrement counter
                self.emit_sub(&name, "1");
                self.indent_level -= 1;
    
                self.indent();
                self.emit_line("}");
                if is_var {
                    self.indent();
                    self.emit_line(&format!("free {}", name));
                }
                if is_var {
                    self.forn_counter -= 1;
                }
            },
            Stmt::While { condition, body } => {
                self.indent();
                self.emit("while ");
                match &condition {
                    Expr::Variable(name) => self.emit(name),
                    Expr::BinaryOp { left, op: _, right: _ } => {
                        if let Expr::Variable(name) = left.as_ref() {
                            self.emit(name);
                        } else {
                            panic!("Complex comparison not supported");
                        }
                    },
                    _ => panic!("Unsupported while condition"),
                }
                self.emit_line(" {");
    
                self.indent_level += 1;
                self.native_loop_depth += 1;
    
                for s in body {
                    self.gen_stmt(s, false);
                }
                self.native_loop_depth -= 1;
                self.indent_level -= 1;
    
                self.indent();
                self.emit_line("}");
            },
            Stmt::ExprStmt(expr) => {
                if let Expr::FunctionCall { name, args } = expr {
                    if let Some(func_def) = self.functions.get(&name).cloned() {
                        if let Stmt::FuncDecl { params, body, .. } = func_def {
                             self.inline_function(params, args, body);
                        }
                    } else {
                        // Function not found, or maybe it's a native/extern?
                        // For now we assume strict simulation or existing funcs.
                        panic!("Undefined function call: {}", name);
                    }
                }
            },
            Stmt::If { condition, then_branch, else_branch } => {
                let folded_cond = self.fold_expr(condition);
                match folded_cond {
                    Expr::BoolLiteral(b) => {
                        if b {
                            for s in then_branch { self.gen_stmt(s, false); }
                        } else if let Some(else_stmts) = else_branch {
                            for s in else_stmts { self.gen_stmt(s, false); }
                        }
                    },
                    Expr::Number(n) => {
                        if n != 0 {
                            for s in then_branch { self.gen_stmt(s, false); }
                        } else if let Some(else_stmts) = else_branch {
                            for s in else_stmts { self.gen_stmt(s, false); }
                        }
                    },
                    _ => {
                        // Runtime IF - convert to single-execution while loop
                        let cond_name = format!("__hbf_if_cond_{}", self.forn_counter);
                        self.forn_counter += 1;
                        self.materialize_to_cell(&cond_name, folded_cond, true);
            
                        if let Some(else_stmts) = else_branch {
                            let else_name = format!("__hbf_else_flag_{}", self.forn_counter);
                            self.forn_counter += 1;
                            self.emit_new(&else_name, "1");
                
                            self.indent();
                            self.emit_line(&format!("while {} {{", cond_name));
                            self.indent_level += 1;
                            for s in then_branch {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&cond_name, "0");
                            self.emit_set(&else_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&cond_name);
                            self.indent();
                            self.emit_line(&format!("while {} {{", else_name));
                            self.indent_level += 1;
                            for s in else_stmts {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&else_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&else_name);
                        } else {
                            self.indent();
                            self.emit_line(&format!("while {} {{", cond_name));
                            self.indent_level += 1;
                            for s in then_branch {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&cond_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&cond_name);
                        }
                    }
                }
            },
            Stmt::Group(stmts) => {
                for s in stmts {
                    self.gen_stmt(s, false);
                }
            },
        }
    }

    pub(super) fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' { self.emit("'\\n'"); }
                else if c == '\t' { self.emit("'\\t'"); }
                else { self.emit(&format!("'{}'", c)); }
            },
            Expr::BoolLiteral(b) => self.emit(&format!("{}", if b { 1 } else { 0 })),
            Expr::Variable(name) => {
                if let Some(val) = self.get_variable(&name) {
                    // Recursively resolve in case of nested virtuals
                    self.gen_expr(val.clone());
                } else {
                    self.emit(&name);
                }
            },
            Expr::FunctionCall { name, .. } => {
                panic!("Function calls in expressions are not supported in simulation-only mode: {}. They must be foldable to constants.", name);
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = &*index {
                    match &*array {
                        Expr::Variable(name) => {
                            // If it's a streaming array, we can return its literal value if known
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if elem_type.is_virtual() {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        self.gen_expr(lit.clone());
                                        return;
                                    }
                                }
                            }
                            self.emit(&self.get_array_var_name(name, *i));
                        },
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                if ch == '\n' { self.emit("'\\n'"); }
                                else if ch == '\t' { self.emit("'\\t'"); }
                                else { self.emit(&format!("'{}'", ch)); }
                            } else { panic!("String index out of bounds: {} in {:?}", i, s); }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                self.gen_expr(el.clone());
                            } else { panic!("Array index out of bounds: {} in {:?}", i, elements); }
                        },
                        _ => panic!("Complex array indexing not supported"),
                    }
                } else {
                    panic!("Only constant array indexing supported");
                }
            },
            Expr::BinaryOp { left, op, right } => {
                self.emit("(");
                self.gen_expr(*left);
                self.emit(&format!(" {} ", match op {
                    Token::Plus => "+",
                    Token::Minus => "-",
                    _ => "?",
                }));
                self.gen_expr(*right);
                self.emit(")");
            },
            Expr::MemberAccess { object, member } => {
                if let Expr::Variable(name) = *object {
                    if member == "length" {
                        if let Some((_, len, _, _)) = self.arrays.get(&name) {
                            self.emit(&len.to_string());
                        } else { panic!("Undefined array: {}", name); }
                    } else { panic!("Unknown member: {}", member); }
                } else { panic!("Member access only supported on variables"); }
            },
            _ => panic!("Complex expression not supported in this context"),
        }
    }

    pub(super) fn gen_expr_simple(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' { self.emit("'\\n'"); }
                else if c == '\t' { self.emit("'\\t'"); }
                else { self.emit(&format!("'{}'", c)); }
            },
            Expr::BoolLiteral(b) => {
                self.emit(&format!("{}", if b { 1 } else { 0 }));
            },
            Expr::Variable(name) => {
                if let Some(val) = self.get_variable(&name) {
                    // Recursively resolve to literal
                    self.gen_expr_simple(val.clone());
                } else {
                    // Assume literal or physical name
                    self.emit(&name);
                }
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = &*index {
                    match &*array {
                        Expr::Variable(name) => {
                            // If it's a streaming array, we can return its literal value if known
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if elem_type.is_virtual() {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        self.gen_expr_simple(lit.clone());
                                        return;
                                    }
                                }
                            }
                            // If it's a physical cell array, we emit the cell name (e.g. arr_1)
                            self.emit(&self.get_array_var_name(name, *i));
                        },
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                if ch == '\n' { self.emit("'\\n'"); }
                                else if ch == '\t' { self.emit("'\\t'"); }
                                else { self.emit(&format!("'{}'", ch)); }
                            } else { panic!("String index out of bounds: {} in {:?}", i, s); }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                self.gen_expr_simple(el.clone());
                            } else { panic!("Array index out of bounds: {} in {:?}", i, elements); }
                        },
                        _ => panic!("Complex array indexing not supported in 'set' context."),
                    }
                } else {
                    panic!("Only constant array indexing supported in 'set' context.");
                }
            },
            Expr::MemberAccess { object, member } => {
                if let Expr::Variable(name) = *object {
                    if member == "length" {
                        if let Some((_, len, _, _)) = self.arrays.get(&name) {
                            self.emit(&len.to_string());
                        } else { panic!("Undefined array: {}", name); }
                    } else { panic!("Unknown member: {}", member); }
                } else { panic!("Member access only supported on variables"); }
            },
            _ => panic!("Only simple expressions (literals) allowed in BFO 'set' context: {:?}", expr),
        }
    }
}
