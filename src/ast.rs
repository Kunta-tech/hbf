
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    StringLiteral(String),
    BinaryOperation {
        left: Box<Expr>,
        op: Token, // Plus, Minus
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Print(Expr),
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
