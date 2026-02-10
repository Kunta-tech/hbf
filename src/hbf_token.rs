
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Void,
    Int,
    Cell,
    Bool,
    String,
    Char,
    For,
    Forn,
    While,
    If,
    Else,
    Func,
    True,
    False,
    
    // Built-in functions
    Putc,
    
    // Identifiers & Literals
    Identifier(String),
    Number(i32),
    StringLiteral(String),
    CharLiteral(char),
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    PlusPlus,       // ++
    MinusMinus,     // --
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    DoubleEquals,   // ==
    NotEquals,      // !=
    AndAnd,         // &&
    OrOr,           // ||
    Equals,         // =
    
    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Semicolon,      // ;
    Comma,          // ,
    Dot,            // .
    
    // Meta
    EOF,
}
