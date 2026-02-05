
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
    Plus,       // +
    Minus,      // -
    PlusPlus,   // ++
    MinusMinus, // --
    Less,       // <
    Greater,    // >
    Equals,     // =
    
    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Semicolon,  // ;
    Comma,      // ,
    Dot,        // .
    
    // Meta
    EOF,
}
