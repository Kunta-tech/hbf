
use crate::hbf_token::Token;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.input.peek() {
            None => Token::EOF,
            Some(&c) => match c {
                '+' => {
                    self.input.next();
                    if let Some(&'+') = self.input.peek() {
                        self.input.next();
                        Token::PlusPlus
                    } else {
                        Token::Plus
                    }
                },
                '-' => {
                    self.input.next();
                    if let Some(&'-') = self.input.peek() {
                        self.input.next();
                        Token::MinusMinus
                    } else {
                        Token::Minus
                    }
                },
                '<' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                },
                '>' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                },
                '=' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::DoubleEquals
                    } else {
                        Token::Equals
                    }
                },
                '!' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::NotEquals
                    } else {
                        panic!("Unexpected character: !");
                    }
                },
                '&' => {
                    self.input.next();
                    if let Some(&'&') = self.input.peek() {
                        self.input.next();
                        Token::AndAnd
                    } else {
                        panic!("Unexpected character: &");
                    }
                },
                '|' => {
                    self.input.next();
                    if let Some(&'|') = self.input.peek() {
                        self.input.next();
                        Token::OrOr
                    } else {
                        panic!("Unexpected character: |");
                    }
                },
                '*' => { self.input.next(); Token::Star },
                '%' => { self.input.next(); Token::Percent },
                '(' => { self.input.next(); Token::LParen },
                ')' => { self.input.next(); Token::RParen },
                '{' => { self.input.next(); Token::LBrace },
                '}' => { self.input.next(); Token::RBrace },
                '[' => { self.input.next(); Token::LBracket },
                ']' => { self.input.next(); Token::RBracket },
                ';' => { self.input.next(); Token::Semicolon },
                ',' => { self.input.next(); Token::Comma },
                '.' => { self.input.next(); Token::Dot },
                '/' => {
                    self.input.next();
                    match self.input.peek() {
                        Some(&'/') => {
                            self.input.next();
                            self.skip_comment();
                            self.next_token()
                        },
                        _ => Token::Slash,
                    }
                },
                '"' => self.read_string(),
                '\'' => self.read_char(),
                c if c.is_alphabetic() || c == '_' => self.read_identifier(),
                c if c.is_numeric() => self.read_number(),
                _ => {
                    let c = self.input.next().unwrap();
                    panic!("Unexpected character: {}", c);
                }
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.input.next();
        }
    }

    fn skip_comment(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c == '\n' {
                break;
            }
            self.input.next();
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.input.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            ident.push(c);
            self.input.next();
        }

        match ident.as_str() {
            "void" => Token::Void,
            "int" => Token::Int,
            "cell" => Token::Cell,
            "bool" => Token::Bool,
            "char" => Token::Char,
            "string" => Token::String,
            "for" => Token::For,
            "forn" => Token::Forn,
            "while" => Token::While,
            "if" => Token::If,
            "else" => Token::Else,
            "func" => Token::Func,
            "true" => Token::True,
            "false" => Token::False,
            "putc" => Token::Putc,
            _ => Token::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        while let Some(&c) = self.input.peek() {
            if !c.is_numeric() {
                break;
            }
            num_str.push(c);
            self.input.next();
        }
        Token::Number(num_str.parse().unwrap())
    }

    fn read_string(&mut self) -> Token {
        self.input.next(); // consume opening "
        let mut s = String::new();
        while let Some(&c) = self.input.peek() {
            if c == '"' {
                self.input.next(); // consume closing "
                return Token::StringLiteral(s);
            }
            if c == '\\' {
                self.input.next(); // consume backslash
                if let Some(&escaped) = self.input.peek() {
                    self.input.next();
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => {
                            s.push('\\');
                            s.push(escaped);
                        }
                    }
                }
            } else {
                s.push(c);
                self.input.next();
            }
        }
        panic!("Unterminated string literal");
    }

    fn read_char(&mut self) -> Token {
        self.input.next(); // consume opening '
        let ch = if let Some(&c) = self.input.peek() {
            self.input.next();
            if c == '\\' {
                // Escape sequence
                if let Some(&escaped) = self.input.peek() {
                    self.input.next();
                    match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => escaped,
                    }
                } else {
                    panic!("Incomplete escape sequence in char literal");
                }
            } else {
                c
            }
        } else {
            panic!("Empty char literal");
        };

        if let Some(&'\'') = self.input.peek() {
            self.input.next(); // consume closing '
            Token::CharLiteral(ch)
        } else {
            panic!("Unterminated char literal");
        }
    }
}
