use super::BFOGenerator;
use crate::hbf_ast::{Expr, Stmt, Type};
use crate::hbf_token::Token;

impl BFOGenerator {
    pub(super) fn gen_stmt(&mut self, stmt: Stmt, is_top_level: bool) {
        match stmt {
            Stmt::VarDecl { var_type, name, value } => {
                let folded = self.fold_expr(value);
                let value_type = self.get_expr_type(&folded);

                if !self.is_compatible(&var_type, &value_type) {
                    panic!("Type mismatch in declaration of '{}': cannot assign {:?} to {:?}", name, value_type, var_type);
                }
    
                match &var_type {
                    Type::Cell => self.materialize_to_cell(&name, folded, true),
                    Type::Array(inner) => {
                        if !inner.is_virtual() {
                            // cell[] is physical: contiguous named cells
                            let elements = match folded {
                                Expr::ArrayLiteral(el) => el,
                                Expr::StringLiteral(s) => s.chars().map(Expr::CharLiteral).collect(),
                                _ => Vec::new(),
                            };

                            if !elements.is_empty() {
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
                let value_type = self.get_expr_type(&folded_val);

                if let Expr::Number(i) = folded_index {
                    let mut type_error = None;
                    if let Some((_, _, elem_type, _)) = self.arrays.get(&name) {
                        if !self.is_compatible(elem_type, &value_type) {
                            type_error = Some((elem_type.clone(), value_type.clone()));
                        }
                    }
                    
                    if let Some((et, vt)) = type_error {
                        panic!("Type mismatch in indexed assignment to array '{}': cannot assign {:?} to element type {:?}", name, vt, et);
                    }

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
                let value_type = self.get_expr_type(&folded);

                // Determine if 'name' is physical or virtual
                if let Some(target_val) = self.get_variable(&name) {
                    let target_type = self.get_expr_type(&target_val);
                    if !self.is_compatible(&target_type, &value_type) {
                        panic!("Type mismatch in assignment to virtual variable '{}': cannot assign {:?} to {:?}", name, value_type, target_type);
                    }

                    if self.native_loop_depth > 0 {
                        eprintln!("WARNING: Modifying virtual variable '{}' inside a native loop. This value will not update in the generated BFO.", name);
                    }
                    self.set_variable(&name, folded);
                } else if self.arrays.contains_key(&name) {
                     let (_, _, elem_type, _) = self.arrays.get(&name).unwrap();
                     let target_type = Type::Array(Box::new(elem_type.clone()));
                     if !self.is_compatible(&target_type, &value_type) {
                         panic!("Type mismatch in assignment to array '{}': cannot assign {:?} to {:?}", name, value_type, target_type);
                     }
                     // For virtual arrays, Assign { name, value } might mean replacing the whole array or it's an error. 
                     // HBF currently treats Assign(name, value) as scalar assign. 
                     // If it's an array, it's usually handled by VarDecl or IndexedAssign.
                     // But let's be strict.
                } else {
                    // Physical cell
                    if !self.is_compatible(&Type::Cell, &value_type) {
                        panic!("Type mismatch in assignment to cell '{}': cannot assign {:?} to Cell", name, value_type);
                    }
                    // println!("Materializing variable {}: {:?}", name, folded);
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
                                self.emit_line(&format!("goto {}", name));
                                self.indent();
                                self.emit_line("print");
                            }
                        } else {
                            self.indent();
                            self.emit_line(&format!("goto {}", name));
                            self.indent();
                            self.emit_line("print");
                        }
                    },
                    Expr::ArrayAccess { array, index } => {
                        if let Expr::Number(i) = index.as_ref() {
                            if let Expr::Variable(array_name) = array.as_ref() {
                                if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                                    if !elem_type.is_virtual() {
                                        let cell_name = self.get_array_var_name(array_name, *i);
                                        self.indent();
                                        self.emit_line(&format!("goto {}", cell_name));
                                        self.indent();
                                        self.emit_line("print");
                                        return;
                                    }
                                }
                            }
                        }
            
                        self.materialize_to_cell("__hbf_tmp", Expr::ArrayAccess { array, index }, true);
                        self.indent();
                        self.emit_line("goto __hbf_tmp");
                        self.indent();
                        self.emit_line("print");
                        self.indent();
                        self.emit_line("free __hbf_tmp");
                    },
                    Expr::Number(n) => {
                        self.materialize_to_cell("__hbf_tmp", Expr::Number(n), true);
                        self.emit_line("print");
                        self.emit_line("free __hbf_tmp");
                    },
                    Expr::CharLiteral(c) => {
                        self.materialize_to_cell("__hbf_tmp", Expr::CharLiteral(c), true);
                        self.emit_line("print");
                        self.emit_line("free __hbf_tmp");
                    },
                    Expr::StringLiteral(s) => {
                        for ch in s.chars() {
                            self.materialize_to_cell("__hbf_tmp", Expr::CharLiteral(ch), true);
                            self.emit_line("print");
                            self.emit_line("free __hbf_tmp");
                        }
                    },
                    Expr::ArrayLiteral(elements) => {
                        for el in elements {
                            self.materialize_to_cell("__hbf_tmp", el.clone(), true);
                            self.emit_line("print");
                            self.emit_line("free __hbf_tmp");
                        }
                    },
                    _ => {
                        self.materialize_to_cell("__hbf_tmp", folded, true);
                        self.indent();
                        self.emit_line("goto __hbf_tmp");
                        self.indent();
                        self.emit_line("print");
                        self.indent();
                        self.emit_line("free __hbf_tmp");
                    }
                }
            },
            Stmt::For { init, condition, update, body } => {
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
                            _ => panic!("For loop condition must be a compile-time constant for unfolding"),
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
                    panic!("Loop unrolling exceeded limit");
                }
                self.pop_scope();
            },
            Stmt::Forn { count, body } => {
                let folded_count = self.fold_expr(count.clone());
                let name = format!("__hbf_forn_{}", self.forn_counter);
                self.forn_counter += 1;
                self.materialize_to_cell(&name, folded_count, true); 
    
                self.indent();
                self.emit_line(&format!("goto {}", name));
                self.indent();
                self.emit_line("loop {");
    
                self.indent_level += 1;
                self.native_loop_depth += 1;
                for s in body {
                    self.gen_stmt(s, false);
                }
                self.native_loop_depth -= 1;
                self.emit_sub(&name, "1");
                self.indent_level -= 1;
    
                self.indent();
                self.emit_line("}");
                self.indent();
                self.emit_line(&format!("free {}", name));
            },
            Stmt::While { condition, body } => {
                let cond_type = self.get_expr_type(&condition);
                if cond_type.is_virtual() {
                    let mut iterations = 0;
                    while iterations < 10000 {
                        let folded = self.fold_expr(condition.clone());
                        let cond_val = match folded {
                            Expr::BoolLiteral(b) => b,
                            Expr::Number(n) => n != 0,
                            _ => panic!("While loop condition with virtual variable must be a compile-time constant"),
                        };
                        if !cond_val { break; }
                        for s in &body {
                            self.gen_stmt(s.clone(), false);
                        }
                        iterations += 1;
                    }
                } else {
                    let cond_name = match &condition {
                        Expr::Variable(name) => name.to_string(),
                        _ => {
                            let tmp = format!("__hbf_while_cond_{}", self.forn_counter);
                            self.forn_counter += 1;
                            self.materialize_to_cell(&tmp, condition, true);
                            tmp
                        }
                    };
                    self.indent();
                    self.emit_line(&format!("goto {}", cond_name));
                    self.indent();
                    self.emit_line("loop {");
                    self.indent_level += 1;
                    self.native_loop_depth += 1;
                    for s in body {
                        self.gen_stmt(s, false);
                    }
                    self.native_loop_depth -= 1;
                    self.indent_level -= 1;
                    self.indent();
                    self.emit_line("}");
                }
            },
            Stmt::ExprStmt(expr) => {
                if let Expr::FunctionCall { name, args } = expr {
                    if let Some(func_def) = self.functions.get(&name).cloned() {
                        if let Stmt::FuncDecl { params, body, .. } = func_def {
                             self.inline_function(params, args, body, None);
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
            Stmt::Return(expr) => {
                let folded = self.fold_expr(expr);
                if let Some(Some(dest)) = self.return_stack.last() {
                    let dest = dest.clone();
                    self.materialize_to_cell(&dest, folded, false);
                }
                // TODO: Early return support would go here
            },
            Stmt::Intrinsic { name, args } => {
                let validate_target = |gen: &mut Self, arg: &Expr, intrinsic_name: &str| -> String {
                    let folded = gen.fold_expr(arg.clone());
                    
                    // Loopholes Fix: Type-aware validation
                    let arg_type = gen.get_expr_type(&folded);
                    if arg_type.is_virtual() {
                        panic!("\n[Error] Invalid target for {}(): The argument must be a mutable 'cell' or 'cell[]' element, but got a virtual type '{:?}' (value: {:?}).\nHint: 'int', 'char', and 'bool' are virtual types and cannot be targets for procedural primitives. Use normal assignment or math instead.\n", intrinsic_name, arg_type, folded);
                    }

                    let target_name = gen.get_var_name(&folded);
                    if target_name.is_empty() {
                        panic!("\n[Error] Invalid target for {}(): The argument must be a mutable cell or array element (e.g. 'a' or 'arr[0]'), but got '{:?}' (folded to '{:?}').\nHint: Literals like '0' or expressions like 'a + b' cannot be used as targets.\n", intrinsic_name, arg, folded);
                    }
                    target_name
                };

                match name {
                    Token::Set => {
                        if args.len() != 2 { panic!("set() requires 2 arguments: target, value"); }
                        let target_name = validate_target(self, &args[0], "set");
                        let value = self.fold_expr(args[1].clone());

                        let val_str = self.get_var_name_or_lit(&value);
                        if val_str.is_empty() {
                            // Loophole Fix: Support non-variable sources via materialization
                            self.materialize_to_cell(&target_name, value, false);
                        } else {
                            self.indent();
                            self.emit_line(&format!("set {} {}", target_name, val_str));
                        }
                    }
                    Token::Copy => {
                        if args.len() != 2 { panic!("copy() requires 2 arguments: dest, src"); }
                        let dest_name = validate_target(self, &args[0], "copy");
                        let src_name = validate_target(self, &args[1], "copy");
                        
                        self.indent();
                        self.emit_line(&format!("set {} {}", dest_name, src_name));
                    }
                    Token::Move => {
                        if args.len() != 2 { panic!("move() requires 2 arguments: dest, src"); }
                        let dest_name = validate_target(self, &args[0], "move");
                        let src_name = validate_target(self, &args[1], "move");
                        
                        self.indent();
                        self.emit_line(&format!("move {} {}", dest_name, src_name));
                    }
                    Token::Clear => {
                        if args.len() != 1 { panic!("clear() requires 1 argument: target"); }
                        let target_name = validate_target(self, &args[0], "clear");
                        self.indent();
                        self.emit_line(&format!("set {} 0", target_name));
                    }
                    Token::Add => {
                        if args.len() != 2 { panic!("add() requires 2 arguments: target, value"); }
                        let target_name = validate_target(self, &args[0], "add");
                        let value = self.fold_expr(args[1].clone());

                        if let Expr::Number(n) = value {
                            if n < 0 {
                                self.indent();
                                self.emit_line(&format!("sub {} {}", target_name, -n));
                                return;
                            }
                        }

                        let val_str = self.get_var_name_or_lit(&value);
                        if val_str.is_empty() {
                            // Loophole Fix: Support non-variable sources via materialization
                            self.materialize_to_cell("__hbf_tmp", value, true);
                            self.indent();
                            self.emit_line(&format!("add {} __hbf_tmp", target_name));
                            self.free_cell("__hbf_tmp");
                        } else {
                            self.indent();
                            self.emit_line(&format!("add {} {}", target_name, val_str));
                        }
                    }
                    Token::Sub => {
                        if args.len() != 2 { panic!("sub() requires 2 arguments: target, value"); }
                        let target_name = validate_target(self, &args[0], "sub");
                        let value = self.fold_expr(args[1].clone());

                        if let Expr::Number(n) = value {
                            if n < 0 {
                                self.indent();
                                self.emit_line(&format!("add {} {}", target_name, -n));
                                return;
                            }
                        }

                        let val_str = self.get_var_name_or_lit(&value);
                        if val_str.is_empty() {
                            // Loophole Fix: Support non-variable sources via materialization
                            self.materialize_to_cell("__hbf_tmp", value, true);
                            self.indent();
                            self.emit_line(&format!("sub {} __hbf_tmp", target_name));
                            self.free_cell("__hbf_tmp");
                        } else {
                            self.indent();
                            self.emit_line(&format!("sub {} {}", target_name, val_str));
                        }
                    }
                    _ => panic!("Unknown intrinsic: {:?}", name),
                }
            }
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
            Expr::Getc => {
                panic!("getc() cannot be generated in simple (runtime) context. It must be materialized.");
            }
            _ => panic!("Only simple expressions (literals) allowed in BFO 'set' context: {:?}", expr),
        }
    }
}
