use super::BFOGenerator;
use crate::hbf_ast::Expr;
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
                // Check for shorthands: A = A + B, A = B + A, A = A - B
                let left_is_name = if let Expr::Variable(v) = left.as_ref() { v == name } else { false };
                let right_is_name = if let Expr::Variable(v) = right.as_ref() { v == name } else { false };

                if left_is_name && *op == Token::Plus {
                    // A = A + right  =>  add A right
                    self.indent();
                    self.emit(&format!("add {} ", name));
                    self.gen_expr_simple(*right.clone());
                    self.emit_line("");
                } else if right_is_name && *op == Token::Plus {
                    // A = left + A  =>  add A left
                    self.indent();
                    self.emit(&format!("add {} ", name));
                    self.gen_expr_simple(*left.clone());
                    self.emit_line("");
                } else if left_is_name && *op == Token::Minus {
                    // A = A - right  =>  sub A right
                    self.indent();
                    self.emit(&format!("sub {} ", name));
                    self.gen_expr_simple(*right.clone());
                    self.emit_line("");
                } else {
                    // General case: clear and rebuild
                    self.emit_new(name, "0");
                    
                    // Add left
                    self.indent();
                    self.emit(&format!("add {} ", name));
                    self.gen_expr_simple(*left.clone());
                    self.emit_line("");
                    
                    // Add/Sub right
                    self.indent();
                    let op_cmd = if *op == Token::Plus { "add" } else { "sub" };
                    self.emit(&format!("{} {} ", op_cmd, name));
                    self.gen_expr_simple(*right.clone());
                    self.emit_line("");
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
}
