use super::BFOGenerator;
use crate::hbf_ast::{Expr, Type};
use crate::hbf_token::Token;

impl BFOGenerator {
    pub(super) fn fold_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Variable(name) => {
                // Substitute variable with its value if known (search scope stack)
                if let Some(value) = self.get_variable(&name) {
                    value
                } else {
                    Expr::Variable(name)
                }
            },
            Expr::BinaryOp { left, op, right } => {
                let left_folded = self.fold_expr(*left);
                let right_folded = self.fold_expr(*right);
                
                // Helper to get numeric value of Number or CharLiteral or BoolLiteral
                let to_num = |e: &Expr| match e {
                    Expr::Number(n) => Some(*n),
                    Expr::CharLiteral(c) => Some(*c as i32),
                    Expr::BoolLiteral(b) => Some(if *b { 1 } else { 0 }),
                    _ => None,
                };

                // Try to evaluate constant expressions
                if let (Some(l), Some(r)) = (to_num(&left_folded), to_num(&right_folded)) {
                    match op {
                        Token::Plus => return Expr::Number(l + r),
                        Token::Minus => return Expr::Number(l - r),
                        Token::Star => return Expr::Number(l * r),
                        Token::Slash => {
                            if r == 0 { panic!("Division by zero in constant folding"); }
                            return Expr::Number(l / r);
                        },
                        Token::Percent => {
                            if r == 0 { panic!("Modulo by zero in constant folding"); }
                            return Expr::Number(l % r);
                        },
                        Token::DoubleEquals => return Expr::BoolLiteral(l == r),
                        Token::NotEquals => return Expr::BoolLiteral(l != r),
                        Token::Less => return Expr::BoolLiteral(l < r),
                        Token::LessEqual => return Expr::BoolLiteral(l <= r),
                        Token::Greater => return Expr::BoolLiteral(l > r),
                        Token::GreaterEqual => return Expr::BoolLiteral(l >= r),
                        Token::AndAnd => return Expr::BoolLiteral((l != 0) && (r != 0)),
                        Token::OrOr => return Expr::BoolLiteral((l != 0) || (r != 0)),
                        _ => {}
                    }
                }
                
                // Can't fold, return the folded operands
                Expr::BinaryOp {
                    left: Box::new(left_folded),
                    op,
                    right: Box::new(right_folded),
                }
            },
            Expr::ArrayAccess { array, index } => {
                let array_folded = self.fold_expr(*array);
                let index_folded = self.fold_expr(*index);

                if let Expr::Number(i) = &index_folded {
                    match &array_folded {
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                return Expr::CharLiteral(ch);
                            }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                return el.clone();
                            }
                        },
                        Expr::Variable(name) => {
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if *elem_type != Type::Cell {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        return lit.clone();
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }

                Expr::ArrayAccess {
                    array: Box::new(array_folded),
                    index: Box::new(index_folded),
                }
            },
            Expr::MemberAccess { object, member } => {
                let object_folded = self.fold_expr(*object);
                if member == "length" {
                    match &object_folded {
                        Expr::StringLiteral(s) => return Expr::Number(s.len() as i32),
                        Expr::ArrayLiteral(elements) => return Expr::Number(elements.len() as i32),
                        Expr::Variable(name) => {
                            if let Some((_, len, _, _)) = self.arrays.get(name) {
                                return Expr::Number(*len as i32);
                            }
                        },
                        _ => {}
                    }
                }
                Expr::MemberAccess {
                    object: Box::new(object_folded),
                    member,
                }
            },
            _ => expr, // Other expressions (CharLiteral, StringLiteral, Number) unchanged
        }
    }
}
