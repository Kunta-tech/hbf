
use crate::token::Token;
use crate::lexer::Lexer;
use crate::ast::{Expr, Stmt, Program};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn eat(&mut self, token: Token) {
        if self.current_token == token {
            self.advance();
        } else {
            panic!("Expected {:?}, got {:?}", token, self.current_token);
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        while self.current_token != Token::EOF {
            statements.push(self.parse_statement());
        }
        Program { statements }
    }

    fn parse_statement(&mut self) -> Stmt {
        match &self.current_token {
            Token::Let => self.parse_let(),
            Token::Print => self.parse_print(),
            Token::While => self.parse_while(),
            Token::Identifier(_) => self.parse_assign(), 
            _ => panic!("Unexpected token at start of statement: {:?}", self.current_token),
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.eat(Token::Let);
        let name = match &self.current_token {
            Token::Identifier(s) => s.clone(),
            _ => panic!("Expected identifier after let"),
        };
        self.advance();
        self.eat(Token::Equals);
        let value = self.parse_expr();
        self.eat(Token::Semicolon);
        Stmt::Let { name, value }
    }

    fn parse_assign(&mut self) -> Stmt {
        let name = match &self.current_token {
            Token::Identifier(s) => s.clone(),
            _ => panic!("Expected identifier"),
        };
        self.advance();
        self.eat(Token::Equals);
        let value = self.parse_expr();
        self.eat(Token::Semicolon);
        Stmt::Assign { name, value }
    }

    fn parse_print(&mut self) -> Stmt {
        self.eat(Token::Print);
        self.eat(Token::LParen);
        let expr = self.parse_expr();
        self.eat(Token::RParen);
        self.eat(Token::Semicolon);
        Stmt::Print(expr)
    }

    fn parse_while(&mut self) -> Stmt {
        self.eat(Token::While);
        self.eat(Token::LParen);
        let condition = self.parse_expr();
        self.eat(Token::RParen);
        self.eat(Token::LBrace);
        let mut body = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            body.push(self.parse_statement());
        }
        self.eat(Token::RBrace);
        Stmt::While { condition, body }
    }

    fn parse_expr(&mut self) -> Expr {
        // For now, handle simple expressions (no precedence or complex nested ops logic yet, just primary + basic binary)
        // Actually, let's implement basic precedence: Term (+,-) Term
        
        let mut left = self.parse_term();

        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_term();
            left = Expr::BinaryOperation {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Number(n) => {
                self.advance();
                Expr::Number(n)
            },
            Token::Identifier(s) => {
                self.advance();
                Expr::Variable(s)
            },
            Token::StringLiteral(s) => {
                self.advance();
                Expr::StringLiteral(s)
            },
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.eat(Token::RParen);
                expr
            },
            _ => panic!("Unexpected token in expression: {:?}", self.current_token),
        }
    }
}
