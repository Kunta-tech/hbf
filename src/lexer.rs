
use crate::token::Token;
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
                '+' => { self.input.next(); Token::Plus },
                '-' => { self.input.next(); Token::Minus },
                '=' => { self.input.next(); Token::Equals },
                '(' => { self.input.next(); Token::LParen },
                ')' => { self.input.next(); Token::RParen },
                '{' => { self.input.next(); Token::LBrace },
                '}' => { self.input.next(); Token::RBrace },
                ';' => { self.input.next(); Token::Semicolon },
                '/' => {
                    self.input.next(); // consume first /
                    if let Some(&'/') = self.input.peek() {
                        self.input.next(); // consume second /
                        self.skip_comment();
                        self.next_token()
                    } else {
                        // TODO: Handle division or error, for now just panic or treat as special
                        panic!("Unexpected character: /");
                    }
                },
                '"' => self.read_string(),
                c if c.is_alphabetic() => self.read_identifier(),
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
            "let" => Token::Let,
            "print" => Token::Print,
            "while" => Token::While,
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
            s.push(c);
            self.input.next();
        }
        panic!("Unterminated string literal");
    }
}
