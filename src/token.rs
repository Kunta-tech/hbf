
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Let,
    Print,
    While,

    // Identifiers & Literals
    Identifier(String),
    Number(i32),
    StringLiteral(String),

    // Symbols
    Plus,       // +
    Minus,      // -
    Equals,     // =
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    Semicolon,  // ;
    
    // Meta
    EOF,
}
