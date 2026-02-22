// BFO Parser - Parses BFO intermediate format into AST

use crate::bfo_ast::*;
use crate::bfo_lexer::{BFOLexer, BFOToken};

pub struct BFOParser<'a> {
    lexer: BFOLexer<'a>,
    current_token: BFOToken,
}

impl<'a> BFOParser<'a> {
    pub fn new(mut lexer: BFOLexer<'a>) -> Self {
        let current_token = lexer.next_token();
        BFOParser {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn eat(&mut self, expected: BFOToken) {
        if self.current_token == expected {
            self.advance();
        } else {
            panic!("Expected {:?}, got {:?}", expected, self.current_token);
        }
    }

    pub fn parse(&mut self) -> BFOProgram {
        let mut items = Vec::new();

        while self.current_token != BFOToken::EOF {
            items.push(self.parse_item());
        }

        BFOProgram { items }
    }

    fn parse_item(&mut self) -> BFOItem {
        match &self.current_token {
            BFOToken::Func => self.parse_function(),
            BFOToken::Include => {
                self.advance();
                if let BFOToken::String(path) = self.current_token.clone() {
                    self.advance();
                    BFOItem::Include(path)
                } else {
                    panic!("Expected string path after include");
                }
            }
            _ => BFOItem::Statement(self.parse_stmt()),
        }
    }

    fn parse_function(&mut self) -> BFOItem {
        self.eat(BFOToken::Func);
        let name = self.parse_identifier();
        self.eat(BFOToken::LParen);
        let params = self.parse_params();
        self.eat(BFOToken::RParen);
        self.eat(BFOToken::LBrace);

        let mut body = Vec::new();
        while self.current_token != BFOToken::RBrace && self.current_token != BFOToken::EOF {
            body.push(self.parse_stmt());
        }

        self.eat(BFOToken::RBrace);

        BFOItem::Function { name, params, body }
    }

    fn parse_params(&mut self) -> Vec<String> {
        let mut params = Vec::new();

        if self.current_token == BFOToken::RParen {
            return params;
        }

        loop {
            params.push(self.parse_identifier());
            if self.current_token == BFOToken::Comma {
                self.advance();
            } else {
                break;
            }
        }

        params
    }

    fn parse_stmt(&mut self) -> BFOStmt {
        match &self.current_token {
            BFOToken::New => {
                self.advance();
                let name = self.parse_identifier();
                let size = match self.parse_value() {
                    BFOValue::Number(n) => n as usize,
                    _ => panic!("Expected size number for new"),
                };
                BFOStmt::New { name, size }
            }
            BFOToken::Free => {
                self.advance();
                let name = self.parse_identifier();
                BFOStmt::Free { name }
            }
            BFOToken::At => {
                self.advance();
                let value = self.parse_value();
                BFOStmt::At(value)
            }
            BFOToken::Goto => {
                self.advance();
                let value = self.parse_value();
                BFOStmt::Goto { value }
            }
            BFOToken::LShift => {
                self.advance();
                let n = match self.parse_value() {
                    BFOValue::Number(n) => n as isize,
                    _ => panic!("Expected shift amount for lshift"),
                };
                BFOStmt::Shift(-n)
            }
            BFOToken::RShift => {
                self.advance();
                let n = match self.parse_value() {
                    BFOValue::Number(n) => n as isize,
                    _ => panic!("Expected shift amount for rshift"),
                };
                BFOStmt::Shift(n)
            }
            BFOToken::Add => {
                self.advance();
                let n = match self.parse_value() {
                    BFOValue::Number(n) => n as i16,
                    BFOValue::Char(c) => c as i16,
                    _ => panic!("Expected numeric value for add"),
                };
                BFOStmt::Modify(n)
            }
            BFOToken::Sub => {
                self.advance();
                let n = match self.parse_value() {
                    BFOValue::Number(n) => n as i16,
                    BFOValue::Char(c) => c as i16,
                    _ => panic!("Expected numeric value for sub"),
                };
                BFOStmt::Modify(-n)
            }
            BFOToken::Set => {
                self.advance();
                let n = match self.parse_value() {
                    BFOValue::Number(n) => n as u8,
                    BFOValue::Char(c) => c as u8,
                    _ => panic!("Expected numeric value for set"),
                };
                BFOStmt::Set(n)
            }
            BFOToken::Ref => {
                self.advance();
                let name = self.parse_identifier();
                let value = self.parse_value();
                BFOStmt::Alias { name, value }
            }
            BFOToken::Print => {
                self.advance();
                BFOStmt::Print
            }
            BFOToken::Scan => {
                self.advance();
                BFOStmt::Scan
            }
            BFOToken::Loop => {
                self.advance();
                self.eat(BFOToken::LBrace);
                let mut body = Vec::new();
                while self.current_token != BFOToken::RBrace && self.current_token != BFOToken::EOF {
                    body.push(self.parse_stmt());
                }
                self.eat(BFOToken::RBrace);
                BFOStmt::Loop(body)
            }
            BFOToken::Identifier(_) => {
                // Function call
                let name = self.parse_identifier();
                self.eat(BFOToken::LParen);
                let args = self.parse_args();
                self.eat(BFOToken::RParen);
                BFOStmt::Call { name, args }
            }
            BFOToken::LBrace => {
                self.advance();
                let mut body = Vec::new();
                while self.current_token != BFOToken::RBrace && self.current_token != BFOToken::EOF {
                    body.push(self.parse_stmt());
                }
                self.eat(BFOToken::RBrace);
                BFOStmt::Block(body)
            }
            _ => panic!("Unexpected token in statement: {:?}", self.current_token),
        }
    }

    fn parse_value(&mut self) -> BFOValue {
        match &self.current_token.clone() {
            BFOToken::Number(n) => {
                let val = *n;
                self.advance();
                BFOValue::Number(val)
            }
            BFOToken::Char(c) => {
                let val = *c;
                self.advance();
                BFOValue::Char(val)
            }
            BFOToken::Identifier(name) => {
                let val = name.clone();
                self.advance();
                let mut offset = 0;
                if self.current_token == BFOToken::Plus {
                    self.advance();
                    if let BFOToken::Number(n) = self.current_token {
                        offset = n as usize;
                        self.advance();
                    } else {
                        panic!("Expected number after + in variable offset");
                    }
                }
                BFOValue::Variable(val, offset)
            }
            _ => panic!("Expected value, got {:?}", self.current_token),
        }
    }

    fn parse_args(&mut self) -> Vec<BFOValue> {
        let mut args = Vec::new();

        if self.current_token == BFOToken::RParen {
            return args;
        }

        loop {
            args.push(self.parse_value());
            if self.current_token == BFOToken::Comma {
                self.advance();
            } else {
                break;
            }
        }

        args
    }

    fn parse_identifier(&mut self) -> String {
        if let BFOToken::Identifier(name) = &self.current_token {
            let result = name.clone();
            self.advance();
            result
        } else {
            panic!("Expected identifier, got {:?}", self.current_token);
        }
    }
}
