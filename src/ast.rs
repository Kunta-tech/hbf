
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Cell,
    String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    CharLiteral(char),
    StringLiteral(String),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        var_type: Type,
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    FuncDecl {
        name: String,
        params: Vec<(Type, String)>,
        return_type: Type,
        body: Vec<Stmt>,
    },
    Putc(Expr),
    For {
        init: Box<Stmt>,
        condition: Expr,
        update: Box<Stmt>,
        body: Vec<Stmt>,
    },
    Forn {
        var_type: Type,
        name: String,
        count: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    ExprStmt(Expr), // For function calls as statements
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
