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
        self.emit_line(&format!("goto {}", name));
        self.indent();
        self.emit_line(&format!("set {}", val));
    }

    pub(super) fn emit_new(&mut self, name: &str, size: &str) {
        self.indent();
        self.emit_line(&format!("new {} {}", name, size));
    }

    pub(super) fn emit_add(&mut self, target: &str, src: &str) {
        // Pointer-oriented BFO doesn't support 'add target src' directly if src is a variable.
        // It's 'goto target; add <literal>'.
        // If src is a variable, we need to handle it via materialization or temp copy.
        // For simplicity in the generator, we'll assume most adds are literals or handled by materialize.
        self.indent();
        self.emit_line(&format!("goto {}", target));
        self.indent();
        self.emit_line(&format!("add {}", src));
    }

    pub(super) fn emit_sub(&mut self, target: &str, src: &str) {
        self.indent();
        self.emit_line(&format!("goto {}", target));
        self.indent();
        self.emit_line(&format!("sub {}", src));
    }

    pub(super) fn materialize_to_cell(&mut self, name: &str, expr: Expr, is_new: bool) {
        match &expr {
            Expr::Number(n) => {
                if is_new {
                    self.emit_new(name, "1");
                }
                self.emit_set(name, &n.to_string());
            }
            Expr::CharLiteral(c) => {
                if is_new {
                    self.emit_new(name, "1");
                }
                self.emit_set(name, &format!("'{}'", c.escape_default()));
            }
            Expr::BoolLiteral(b) => {
                if is_new {
                    self.emit_new(name, "1");
                }
                self.emit_set(name, if *b { "1" } else { "0" });
            }
            Expr::Variable(v) => {
                if name == v { return; }
                // In pointer-oriented, copy is goto src; loop { goto dest; add 1; goto temp; add 1; goto src; sub 1 } ...
                // This is better handled by a 'copy' macro or direct IR if we had it.
                // For now, let's stick to the high-level intent but emit proper instructions.
                if is_new {
                    self.emit_new(name, "1");
                }
                self.indent();
                self.emit_line(&format!("goto {}", v));
                self.emit_line("    loop {");
                self.emit_line(&format!("        goto {}", name));
                self.emit_line("        add 1");
                self.emit_line("        new __hbf_copy_tmp 1");
                self.emit_line("        goto __hbf_copy_tmp");
                self.emit_line("        add 1");
                self.emit_line(&format!("        goto {}", v));
                self.emit_line("        sub 1");
                self.emit_line("    }");
                self.emit_line("    goto __hbf_copy_tmp");
                self.emit_line("    loop {");
                self.emit_line(&format!("        goto {}", v));
                self.emit_line("        add 1");
                self.emit_line("        goto __hbf_copy_tmp");
                self.emit_line("        sub 1");
                self.emit_line("    }");
                self.emit_line("    free __hbf_copy_tmp");
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = index.as_ref() {
                    if let Expr::Variable(array_name) = array.as_ref() {
                        if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                            if !elem_type.is_virtual() {
                                let cell_name = self.get_array_var_name(array_name, *i);
                                self.material_variable_to_cell(name, &cell_name, is_new);
                                return;
                            }
                        }
                    }
                }
                
                self.indent();
                if is_new {
                    self.emit_line(&format!("new {} 1", name));
                }
                self.emit_line(&format!("goto {}", name));
                self.indent();
                self.emit("set ");
                self.gen_expr_simple(expr);
                self.emit_line("");
            },
            Expr::BinaryOp { left, op, right } => {
                let left_folded = self.fold_expr(*left.clone());
                let right_folded = self.fold_expr(*right.clone());

                self.materialize_to_cell(name, left_folded, is_new);
                
                match op {
                    Token::Plus => {
                        if let Expr::Number(n) = right_folded {
                            self.emit_add(name, &n.to_string());
                        } else {
                            let val_str = self.get_var_name_or_lit(&right_folded);
                            if val_str.is_empty() {
                                self.materialize_to_cell("__hbf_tmp", right_folded, true);
                                self.material_add_variable_to_cell(name, "__hbf_tmp");
                                self.free_cell("__hbf_tmp");
                            } else {
                                self.material_add_variable_to_cell(name, &val_str);
                            }
                        }
                    }
                    Token::Minus => {
                        if let Expr::Number(n) = right_folded {
                            self.emit_sub(name, &n.to_string());
                        } else {
                            let val_str = self.get_var_name_or_lit(&right_folded);
                            if val_str.is_empty() {
                                self.materialize_to_cell("__hbf_tmp", right_folded, true);
                                self.material_sub_variable_from_cell(name, "__hbf_tmp");
                                self.free_cell("__hbf_tmp");
                            } else {
                                self.material_sub_variable_from_cell(name, &val_str);
                            }
                        }
                    }
                    _ => panic!("Complex cell-type math ({:?}) is still restricted.", op),
                }
            },
            Expr::Getc => {
                if is_new {
                    self.emit_new(name, "1");
                }
                self.indent();
                self.emit_line(&format!("goto {}", name));
                self.indent();
                self.emit_line("scan");
            }
            Expr::FunctionCall { name: func_name, args } => {
                if let Some(func_def) = self.functions.get(func_name).cloned() {
                    if let Stmt::FuncDecl { params, body, .. } = func_def {
                        if is_new {
                            self.emit_new(name, "1");
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
                    self.emit_line(&format!("new {} 1", name));
                }
                self.emit_line(&format!("goto {}", name));
                self.indent();
                self.emit("set ");
                self.gen_expr_simple(expr);
                self.emit_line("");
            }
        }
    }

    pub(super) fn free_cell(&mut self, name: &str) {
        self.indent();
        self.emit_line(&format!("free {}", name));
    }

    fn material_variable_to_cell(&mut self, dest: &str, src: &str, is_new: bool) {
        if dest == src { return; }
        if is_new { self.emit_new(dest, "1"); }
        self.indent();
        self.emit_line(&format!("goto {}", dest));
        self.emit_line("    set 0");
        self.indent();
        self.emit_line(&format!("goto {}", src));
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", dest));
        self.emit_line("        add 1");
        self.emit_line("        new __hbf_copy_tmp 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        add 1");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    goto __hbf_copy_tmp");
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        add 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    free __hbf_copy_tmp");
    }

    fn material_add_variable_to_cell(&mut self, dest: &str, src: &str) {
        self.indent();
        self.emit_line(&format!("goto {}", src));
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", dest));
        self.emit_line("        add 1");
        self.emit_line("        new __hbf_copy_tmp 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        add 1");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    goto __hbf_copy_tmp");
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        add 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    free __hbf_copy_tmp");
    }

    fn material_sub_variable_from_cell(&mut self, dest: &str, src: &str) {
        self.indent();
        self.emit_line(&format!("goto {}", src));
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", dest));
        self.emit_line("        sub 1");
        self.emit_line("        new __hbf_copy_tmp 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        add 1");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    goto __hbf_copy_tmp");
        self.emit_line("    loop {");
        self.emit_line(&format!("        goto {}", src));
        self.emit_line("        add 1");
        self.emit_line("        goto __hbf_copy_tmp");
        self.emit_line("        sub 1");
        self.emit_line("    }");
        self.emit_line("    free __hbf_copy_tmp");
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
