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
            BFOToken::Set => {
                self.advance();
                let name = self.parse_identifier();
                let value = self.parse_value();
                BFOStmt::Set { name, value }
            }
            BFOToken::Add => {
                self.advance();
                let name = self.parse_identifier();
                let value = self.parse_value();
                BFOStmt::Add { name, value }
            }
            BFOToken::Sub => {
                self.advance();
                let name = self.parse_identifier();
                let value = self.parse_value();
                BFOStmt::Sub { name, value }
            }
            BFOToken::Print => {
                self.advance();
                let value = self.parse_value();
                BFOStmt::Print { value }
            }
            BFOToken::While => {
                self.advance();
                let condition = self.parse_identifier();
                self.eat(BFOToken::LBrace);

                let mut body = Vec::new();
                while self.current_token != BFOToken::RBrace && self.current_token != BFOToken::EOF {
                    body.push(self.parse_stmt());
                }

                self.eat(BFOToken::RBrace);
                BFOStmt::While { condition, body }
            }
            BFOToken::Identifier(_) => {
                // Function call
                let name = self.parse_identifier();
                self.eat(BFOToken::LParen);
                let args = self.parse_args();
                self.eat(BFOToken::RParen);
                BFOStmt::Call { name, args }
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
                BFOValue::Variable(val)
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
