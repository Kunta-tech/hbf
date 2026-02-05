
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Cell,
    Bool,
    Char,
    Array(Box<Type>),
}

impl Type {
    pub fn is_virtual(&self) -> bool {
        match self {
            Type::Int | Type::Bool | Type::Char => true,
            Type::Array(inner) => inner.is_virtual(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    BoolLiteral(bool),
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
    ArrayLiteral(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        var_type: Type,
        name: String,
        value: Expr,
    },
    Increment {
        name: String,
    },
    Decrement {
        name: String,
    },
    Assign {
        name: String,
        value: Expr,
    },
    IndexedAssign {
        name: String,
        index: Expr,
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
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    Forn {
        count: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    Group(Vec<Stmt>), // For grouping statements (like multi-var declaration) without scope
    ExprStmt(Expr), // For function calls as statements
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
