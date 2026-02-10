// BFO Lexer - Tokenizes BFO intermediate format

#[derive(Debug, Clone, PartialEq)]
pub enum BFOToken {
    // Keywords
    Func,
    New,
    Set,
    Add,
    Sub,
    Print,
    While,
    Free,
    
    // Literals
    Number(i32),
    Char(char),
    Identifier(String),
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    
    EOF,
}

pub struct BFOLexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> BFOLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        BFOLexer {
            input: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> BFOToken {
        self.skip_whitespace();

        match self.input.peek() {
            None => BFOToken::EOF,
            Some(&ch) => match ch {
                '(' => {
                    self.input.next();
                    BFOToken::LParen
                }
                ')' => {
                    self.input.next();
                    BFOToken::RParen
                }
                '{' => {
                    self.input.next();
                    BFOToken::LBrace
                }
                '}' => {
                    self.input.next();
                    BFOToken::RBrace
                }
                ',' => {
                    self.input.next();
                    BFOToken::Comma
                }
                '\'' => self.read_char_literal(),
                ';' => {
                    // Skip comments
                    while let Some(&c) = self.input.peek() {
                        self.input.next();
                        if c == '\n' {
                            break;
                        }
                    }
                    self.next_token()
                }
                '0'..='9' | '-' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
                _ => panic!("Unexpected character in BFO: {}", ch),
            },
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> BFOToken {
        let mut num_str = String::new();
        
        // Handle negative sign
        if let Some(&'-') = self.input.peek() {
            num_str.push('-');
            self.input.next();
        }
        
        while let Some(&ch) = self.input.peek() {
            if ch.is_numeric() {
                num_str.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        BFOToken::Number(num_str.parse().expect("Invalid number"))
    }

    fn read_char_literal(&mut self) -> BFOToken {
        self.input.next(); // Skip opening '
        
        let ch = if let Some(&c) = self.input.peek() {
            self.input.next();
            if c == '\\' {
                // Escape sequence
                if let Some(&next) = self.input.peek() {
                    self.input.next();
                    match next {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => panic!("Unknown escape sequence: \\{}", next),
                    }
                } else {
                    panic!("Incomplete escape sequence");
                }
            } else {
                c
            }
        } else {
            panic!("Incomplete char literal");
        };
        
        if let Some(&'\'') = self.input.peek() {
            self.input.next();
        } else {
            panic!("Expected closing ' for char literal");
        }
        
        BFOToken::Char(ch)
    }

    fn read_identifier(&mut self) -> BFOToken {
        let mut ident = String::new();
        
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        match ident.as_str() {
            "func" => BFOToken::Func,
            "set" => BFOToken::Set,
            "new" => BFOToken::New,
            "add" => BFOToken::Add,
            "sub" => BFOToken::Sub,
            "print" => BFOToken::Print,
            "while" => BFOToken::While,
            "free" => BFOToken::Free,
            _ => BFOToken::Identifier(ident),
        }
    }
}
