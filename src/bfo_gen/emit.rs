use super::BFOGenerator;
use crate::hbf_ast::{Expr, Stmt, Type};
use crate::hbf_token::Token;

impl BFOGenerator {
    pub(super) fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(super) fn emit_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    pub(super) fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    pub(super) fn emit_set(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("set {} {}", name, val));
    }

    pub(super) fn emit_new(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("new {} {}", name, val));
    }

    pub(super) fn emit_add(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("add {} {}", name, val));
    }

    pub(super) fn emit_sub(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("sub {} {}", name, val));
    }

    pub(super) fn materialize_to_cell(&mut self, name: &str, expr: Expr, is_new: bool) {
        match &expr {
            Expr::Number(n) => {
                if is_new {
                    self.emit_new(name, &n.to_string());
                } else {
                    self.emit_set(name, &n.to_string());
                }
            }
            Expr::CharLiteral(c) => {
                if is_new {
                    self.emit_new(name, &format!("'{}'", c.escape_default()));
                } else {
                    self.emit_set(name, &format!("'{}'", c.escape_default()));
                }
            }
            Expr::BoolLiteral(b) => {
                if is_new {
                    self.emit_new(name, if *b { "1" } else { "0" });
                } else {
                    self.emit_set(name, if *b { "1" } else { "0" });
                }
            }
            Expr::Variable(v) => {
                if name == v { return; }
                if is_new {
                    self.emit_new(name, "0");
                } else {
                    self.emit_set(name, "0");
                }
                self.emit_add(name, v);
            },
            Expr::ArrayAccess { array, index } => {
                // If it's a physical array access, we treat it as a variable for the copy
                if let Expr::Number(i) = index.as_ref() {
                    if let Expr::Variable(array_name) = array.as_ref() {
                        if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                            if !elem_type.is_virtual() {
                                let cell_name = self.get_array_var_name(array_name, *i);
                                // Initialize the target variable before adding
                                if is_new {
                                    self.emit_new(name, "0");
                                } else {
                                    self.emit_set(name, "0");
                                }
                                self.emit_add(name, &cell_name);
                                return;
                            }
                        }
                    }
                }
                
                // Otherwise fall back to gen_expr_simple
                self.indent();
                if is_new {
                    self.emit(&format!("new {} ", name));
                } else {
                    self.emit(&format!("set {} ", name));
                }
                self.gen_expr_simple(expr);
                self.emit_line("");
            },
            Expr::BinaryOp { left, op, right } => {
                let left_folded = self.fold_expr(*left.clone());
                let right_folded = self.fold_expr(*right.clone());

                // Loopholes Fix: Allow auto-materialization for procedural primitive arguments
                // 1. Initialize result with left operand
                self.materialize_to_cell(name, left_folded, is_new);
                
                // 2. Apply operator with right operand
                match op {
                    Token::Plus => {
                        let val_str = self.get_var_name_or_lit(&right_folded);
                        if val_str.is_empty() {
                            self.materialize_to_cell("__hbf_tmp", right_folded, true);
                            self.emit_add(name, "__hbf_tmp");
                            self.free_cell("__hbf_tmp");
                        } else {
                            self.emit_add(name, &val_str);
                        }
                    }
                    Token::Minus => {
                        let val_str = self.get_var_name_or_lit(&right_folded);
                        if val_str.is_empty() {
                            self.materialize_to_cell("__hbf_tmp", right_folded, true);
                            self.emit_sub(name, "__hbf_tmp");
                            self.free_cell("__hbf_tmp");
                        } else {
                            self.emit_sub(name, &val_str);
                        }
                    }
                    _ => panic!("Complex cell-type math ({:?}) is still restricted. Use Procedural Primitives for efficiency control.", op),
                }
            },
            Expr::Getc => {
                if is_new {
                    self.emit_new(name, "0");
                }
                self.indent();
                self.emit_line(&format!("scan {}", name));
            }
            Expr::FunctionCall { name: func_name, args } => {
                if let Some(func_def) = self.functions.get(func_name).cloned() {
                    if let Stmt::FuncDecl { params, body, .. } = func_def {
                        if is_new {
                            self.emit_new(name, "0");
                        }
                        self.inline_function(params, args.clone(), body, Some(name.to_string()));
                    }
                } else {
                    panic!("Undefined function call: {}", func_name);
                }
            },
            _ => {
                self.indent();
                if is_new {
                    self.emit(&format!("new {} ", name));
                } else {
                    self.emit(&format!("set {} ", name));
                }
                self.gen_expr_simple(expr);
                self.emit_line("");
            }
        }
    }

    pub(super) fn free_cell(&mut self, name: &str) {
        self.indent();
        self.emit_line(&format!("free {}", name));
    }
    pub(super) fn get_var_name(&self, val: &Expr) -> String {
        match val {
            Expr::Variable(name) => name.to_string(),
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = index.as_ref() {
                    if let Expr::Variable(array_name) = array.as_ref() {
                        if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                            if !elem_type.is_virtual() {
                                return self.get_array_var_name(array_name, *i);
                            }
                        }
                    }
                }
                return "".to_string();
            }
            Expr::Getc => return "".to_string(),
            _ => return "".to_string(),
        }
    }

    pub(super) fn get_var_name_or_lit(&self, val: &Expr) -> String {
        match val {
            Expr::Number(n) => n.to_string(),
            Expr::CharLiteral(c) => format!("'{}'", c.escape_default()),
            Expr::BoolLiteral(b) => if *b { "1".to_string() } else { "0".to_string() },
            Expr::Getc => "".to_string(),
            _ => self.get_var_name(val),
        }
    }
    pub(super) fn get_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_) => Type::Int,
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::CharLiteral(_) => Type::Char,
            Expr::StringLiteral(_) => Type::String,
            Expr::Variable(name) => {
                if let Some(folded) = self.get_variable(name) {
                    return self.get_expr_type(&folded);
                } else if let Some((_, _, elem_type, _)) = self.arrays.get(name) {
                    return Type::Array(Box::new(elem_type.clone()));
                } else {
                    // Not in virtual variables or arrays -> must be a physical Cell
                    Type::Cell
                }
            }
            Expr::BinaryOp { left, op, right } => {
                if *op == Token::Star {
                    let lt = self.get_expr_type(left);
                    let rt = self.get_expr_type(right);
                    if matches!(lt, Type::Array(_) | Type::String) { return lt; }
                    if matches!(rt, Type::Array(_) | Type::String) { return rt; }
                }
                Type::Int 
            }
            Expr::ArrayAccess { array, .. } => {
                let array_type = self.get_expr_type(array);
                match array_type {
                    Type::Array(inner) => *inner,
                    Type::String => Type::Char,
                    _ => Type::Cell, // Fallback
                }
            }
            Expr::FunctionCall { name, .. } => {
                if let Some(Stmt::FuncDecl { return_type, .. }) = self.functions.get(name) {
                    return_type.clone()
                } else {
                    Type::Cell // Fallback for unknown or native functions
                }
            }
            Expr::ArrayLiteral(elements) => {
                if let Some(first) = elements.get(0) {
                    Type::Array(Box::new(self.get_expr_type(first)))
                } else {
                    Type::Array(Box::new(Type::Int)) // Default for empty
                }
            }
            Expr::Getc => Type::Cell,
            Expr::MemberAccess { member, .. } => {
                if member == "length" { Type::Int } else { Type::Cell }
            }
        }
    }

    pub(super) fn is_compatible(&self, target: &Type, value: &Type) -> bool {
        // println!("DEBUG: is_compatible target={:?} value={:?}", target, value);
        if target == value { return true; }

        let is_scalar = |t: &Type| matches!(t, Type::Int | Type::Char | Type::Bool);
        
        // If target is Cell, it accepts any virtual scalar
        if *target == Type::Cell && is_scalar(value) { return true; }

        // If both are virtual scalars, they are compatible
        if is_scalar(target) && is_scalar(value) { return true; }

        // Nesting strictness for arrays
        match (target, value) {
            (Type::Array(t_inner), Type::Array(v_inner)) => self.is_compatible(t_inner, v_inner),
            (Type::Array(inner), Type::String) | (Type::String, Type::Array(inner)) => {
                self.is_compatible(inner, &Type::Char)
            }
            _ => false
        }
    }
}
