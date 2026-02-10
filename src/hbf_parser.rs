
use crate::hbf_token::Token;
use crate::hbf_lexer::Lexer;
use crate::hbf_ast::{Expr, Stmt, Program, Type};

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
            statements.push(self.parse_top_level());
        }
        Program { statements }
    }

    fn parse_top_level(&mut self) -> Stmt {
        // Top level can be function declarations, variable declarations, or function calls
        match &self.current_token {
            Token::Void => self.parse_function_decl(),
            Token::Int | Token::Cell | Token::Bool | Token::Char | Token::String => {
                // Could be function or variable
                let var_type = self.parse_type();
                if let Token::Identifier(name) = &self.current_token {
                    let name_clone = name.clone();
                    self.advance();
                    if self.current_token == Token::LParen {
                        // It's a function
                        panic!("Non-void functions not yet supported");
                    } else {
                        // It's a variable declaration (or list of them)
                        let stmt = self.parse_var_list(var_type, Some(name_clone));
                        self.eat(Token::Semicolon);
                        stmt
                    }
                } else {
                    panic!("Expected identifier after type");
                }
            },
            Token::If => self.parse_if(),
            Token::Identifier(_) => {
                let expr = self.parse_expr();
                if self.current_token == Token::Equals {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::Equals);
                            let value = self.parse_expr();
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name, value }
                        },
                        Expr::ArrayAccess { array, index } => {
                            if let Expr::Variable(array_name) = *array {
                                self.eat(Token::Equals);
                                let value = self.parse_expr();
                                self.eat(Token::Semicolon);
                                Stmt::IndexedAssign { name: array_name, index: *index, value }
                            } else {
                                panic!("Only variable arrays can be assigned to");
                            }
                        },
                        _ => panic!("Invalid assignment target"),
                    }
                } else if self.current_token == Token::PlusPlus {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::PlusPlus);
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name.clone(),
                                    value: Expr::BinaryOp { left: Box::new(Expr::Variable(var_name)),
                                    op: Token::Plus,
                                    right: Box::new(Expr::Number(1))
                                    }
                                }
                        }
                        _ => panic!("++ can only be applied to variables"),
                    }
                } else if self.current_token == Token::MinusMinus {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::MinusMinus);
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name.clone(),
                                    value: Expr::BinaryOp { left: Box::new(Expr::Variable(var_name)),
                                    op: Token::Minus,
                                    right: Box::new(Expr::Number(1))
                                    }
                                }
                        }
                        _ => panic!("-- can only be applied to variables"),
                    }
                } else {
                    self.eat(Token::Semicolon);
                    Stmt::ExprStmt(expr)
                }
            },
            Token::For => self.parse_for(),
            Token::Forn => self.parse_forn(),
            Token::While => self.parse_while(),
            Token::Putc => self.parse_putc(),
            _ => panic!("Unexpected token at top level: {:?}", self.current_token),
        }
    }

    fn parse_function_decl(&mut self) -> Stmt {
        self.eat(Token::Void);
        let name = if let Token::Identifier(n) = &self.current_token {
            n.clone()
        } else {
            panic!("Expected function name");
        };
        self.advance();
        
        self.eat(Token::LParen);
        let params = self.parse_params();
        self.eat(Token::RParen);
        
        self.eat(Token::LBrace);
        let mut body = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            body.push(self.parse_statement());
        }
        self.eat(Token::RBrace);
        
        Stmt::FuncDecl {
            name,
            params,
            return_type: Type::Void,
            body,
        }
    }

    fn parse_params(&mut self) -> Vec<(Type, String)> {
        let mut params = Vec::new();
        
        if self.current_token == Token::RParen {
            return params;
        }
        
        loop {
            let param_type = self.parse_type();
            let name = if let Token::Identifier(n) = &self.current_token {
                n.clone()
            } else {
                panic!("Expected parameter name");
            };
            self.advance();
            params.push((param_type, name));
            
            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        params
    }

    fn parse_type(&mut self) -> Type {
        let mut t = match &self.current_token {
            Token::Void => Type::Void,
            Token::Int => Type::Int,
            Token::Cell => Type::Cell,
            Token::Bool => Type::Bool,
            Token::Char => Type::Char,
            Token::String => Type::Array(Box::new(Type::Char)), // char[]
            _ => panic!("Expected type, got {:?}", self.current_token),
        };
        self.advance();
        
        // Handle Java-style array type syntax: type[] a;
        while self.current_token == Token::LBracket {
            self.eat(Token::LBracket);
            self.eat(Token::RBracket);
            t = Type::Array(Box::new(t));
        }
        
        t
    }

    // Helper to parse comma-separated variable list: type a, b=1, c[] ...
    fn parse_var_list(&mut self, base_type: Type, mut first_name: Option<String>) -> Stmt {
        let mut stmts = Vec::new();
        
        loop {
            let name = if let Some(n) = first_name.take() {
                n
            } else {
                if let Token::Identifier(n) = &self.current_token {
                    let n = n.clone();
                    self.advance();
                    n
                } else {
                    panic!("Expected variable name");
                }
            };
            
            // Handle C-style array syntax: type a[];
            let mut current_type = base_type.clone();
            while self.current_token == Token::LBracket {
                self.eat(Token::LBracket);
                self.eat(Token::RBracket);
                current_type = Type::Array(Box::new(current_type));
            }

            let value = if self.current_token == Token::Equals {
                self.eat(Token::Equals);
                self.parse_expr()
            } else {
                // Default initialization
                match &current_type {
                    Type::Array(_) => Expr::ArrayLiteral(vec![]), // Default empty array
                    _ => Expr::Number(0),
                }
            };
            
            stmts.push(Stmt::VarDecl { var_type: current_type, name, value });
            
            if self.current_token == Token::Comma {
                self.eat(Token::Comma);
            } else {
                break;
            }
        }
        
        if stmts.len() == 1 {
            stmts.pop().unwrap()
        } else {
            Stmt::Group(stmts)
        }
    }

    fn parse_if(&mut self) -> Stmt {
        self.eat(Token::If);
        self.eat(Token::LParen);
        let condition = self.parse_expr();
        self.eat(Token::RParen);
        
        self.eat(Token::LBrace);
        let mut then_branch = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            then_branch.push(self.parse_statement());
        }
        self.eat(Token::RBrace);
        
        let else_branch = if self.current_token == Token::Else {
            self.eat(Token::Else);
            if self.current_token == Token::If {
                // else if ... -> Treat as a single statement in the else block
                let if_stmt = self.parse_if();
                Some(vec![if_stmt])
            } else {
                // else { ... }
                self.eat(Token::LBrace);
                let mut else_stmts = Vec::new();
                while self.current_token != Token::RBrace && self.current_token != Token::EOF {
                    else_stmts.push(self.parse_statement());
                }
                self.eat(Token::RBrace);
                Some(else_stmts)
            }
        } else {
            None
        };
        
        Stmt::If { condition, then_branch, else_branch }
    }


    fn parse_statement(&mut self) -> Stmt {
        match &self.current_token {
            Token::Int | Token::Cell | Token::Bool | Token::Char | Token::String => self.parse_var_decl(),
            Token::For => self.parse_for(),
            Token::Forn => self.parse_forn(),
            Token::While => self.parse_while(),
            Token::If => self.parse_if(),
            Token::Putc => self.parse_putc(),
            Token::Identifier(_) => {
                // Peek at next token using a temporary variable if possible
                // Since our parser doesn't have peek, we'll use a match on the current state
                let _name = if let Token::Identifier(n) = &self.current_token {
                    n.clone()
                } else {
                    unreachable!()
                };
                
                // If the statement starts with identifier, it must be either an assignment or a function call
                // Let's use parse_expr() for everything and then check if it was an assignment
                // Actually, our AST has Assign as a Statement, not an Expression.
                // So let's just peek ahead properly.
                
                // Save current state for simpler handling
                let expr = self.parse_expr();
                
                if self.current_token == Token::Equals {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::Equals);
                            let value = self.parse_expr();
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name, value }
                        },
                        Expr::ArrayAccess { array, index } => {
                            if let Expr::Variable(array_name) = *array {
                                self.eat(Token::Equals);
                                let value = self.parse_expr();
                                self.eat(Token::Semicolon);
                                Stmt::IndexedAssign { name: array_name, index: *index, value }
                            } else {
                                panic!("Only variable arrays can be assigned to");
                            }
                        },
                        _ => panic!("Invalid assignment target"),
                    }
                } else if self.current_token == Token::PlusPlus {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::PlusPlus);
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name.clone(),
                                    value: Expr::BinaryOp { left: Box::new(Expr::Variable(var_name)),
                                    op: Token::Plus,
                                    right: Box::new(Expr::Number(1))
                                    }
                                }
                        }
                        _ => panic!("++ can only be applied to variables"),
                    }
                } else if self.current_token == Token::MinusMinus {
                    match expr {
                        Expr::Variable(var_name) => {
                            self.eat(Token::MinusMinus);
                            self.eat(Token::Semicolon);
                            Stmt::Assign { name: var_name.clone(),
                                    value: Expr::BinaryOp { left: Box::new(Expr::Variable(var_name)),
                                    op: Token::Minus,
                                    right: Box::new(Expr::Number(1))
                                    }
                                }
                        }
                        _ => panic!("-- can only be applied to variables"),
                    }
                } else {
                    // Was a function call or just an expression statement
                    self.eat(Token::Semicolon);
                    Stmt::ExprStmt(expr)
                }
            },
            _ => panic!("Unexpected token at start of statement: {:?}", self.current_token),
        }
    }

    fn parse_var_decl(&mut self) -> Stmt {
        let var_type = self.parse_type();
        let stmt = self.parse_var_list(var_type, None);
        self.eat(Token::Semicolon);
        stmt
    }

    fn parse_for(&mut self) -> Stmt {
        self.eat(Token::For);
        self.eat(Token::LParen);
        
        let init = if self.current_token == Token::Semicolon {
            None
        } else {
            // Check if it's a declaration or assignment
            match &self.current_token {
                Token::Int | Token::Cell | Token::Bool | Token::Char | Token::String => {
                    Some(Box::new(self.parse_var_decl_no_semi()))
                },
                _ => Some(Box::new(self.parse_assignment_no_semi())),
            }
        };
        self.eat(Token::Semicolon);
        
        let condition = if self.current_token == Token::Semicolon {
            None
        } else {
            Some(self.parse_expr())
        };
        self.eat(Token::Semicolon);
        
        let update = if self.current_token == Token::RParen {
            None
        } else {
            Some(Box::new(self.parse_assignment_no_semi()))
        };
        self.eat(Token::RParen);
        
        self.eat(Token::LBrace);
        let mut body = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            body.push(self.parse_statement());
        }
        self.eat(Token::RBrace);
        
        Stmt::For { init, condition, update, body }
    }

    fn parse_forn(&mut self) -> Stmt {
        self.eat(Token::Forn);
        self.eat(Token::LParen);
        
        let count = self.parse_expr();
        self.eat(Token::RParen);
        
        self.eat(Token::LBrace);
        let mut body = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            body.push(self.parse_statement());
        }
        self.eat(Token::RBrace);
        
        Stmt::Forn { count, body }
    }

    fn parse_var_decl_no_semi(&mut self) -> Stmt {
        let var_type = self.parse_type();
        self.parse_var_list(var_type, None)
    }

    fn parse_assignment_no_semi(&mut self) -> Stmt {
        let name = if let Token::Identifier(n) = &self.current_token {
            n.clone()
        } else {
            panic!("Expected identifier");
        };
        self.advance();
        
        // Handle i++, i--
        if self.current_token == Token::PlusPlus {
            self.advance();
            // i++ becomes i = i + 1
            return Stmt::Assign {
                name: name.clone(),
                value: Expr::BinaryOp {
                    left: Box::new(Expr::Variable(name)),
                    op: Token::Plus,
                    right: Box::new(Expr::Number(1)),
                },
            };
        } else if self.current_token == Token::MinusMinus {
            self.advance();
            // i-- becomes i = i - 1
            return Stmt::Assign {
                name: name.clone(),
                value: Expr::BinaryOp {
                    left: Box::new(Expr::Variable(name)),
                    op: Token::Minus,
                    right: Box::new(Expr::Number(1)),
                },
            };
        }
        
        self.eat(Token::Equals);
        let value = self.parse_expr();
        Stmt::Assign { name, value }
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

    fn parse_putc(&mut self) -> Stmt {
        self.eat(Token::Putc);
        self.eat(Token::LParen);
        let expr = self.parse_expr();
        self.eat(Token::RParen);
        self.eat(Token::Semicolon);
        Stmt::Putc(expr)
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expr {
        let mut left = self.parse_logical_and();
        
        while self.current_token == Token::OrOr {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_logical_and();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_logical_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        
        while self.current_token == Token::AndAnd {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_equality();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        
        while self.current_token == Token::DoubleEquals || self.current_token == Token::NotEquals {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_comparison();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_additive();
        
        while matches!(self.current_token, Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual) {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_additive();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        
        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_multiplicative();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_postfix();
        
        while matches!(self.current_token, Token::Star | Token::Slash | Token::Percent) {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_postfix();
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        left
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        
        loop {
            match &self.current_token {
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr();
                    self.eat(Token::RBracket);
                    expr = Expr::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                },
                Token::Dot => {
                    self.advance();
                    let member = if let Token::Identifier(m) = &self.current_token {
                        m.clone()
                    } else {
                        panic!("Expected member name after '.'");
                    };
                    self.advance();
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                    };
                },
                Token::LParen => {
                    // Function call
                    if let Expr::Variable(name) = expr {
                        self.advance();
                        let args = self.parse_args();
                        self.eat(Token::RParen);
                        expr = Expr::FunctionCall { name, args };
                    } else {
                        panic!("Invalid function call");
                    }
                },
                _ => break,
            }
        }
        
        expr
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        
        if self.current_token == Token::RParen {
            return args;
        }
        
        loop {
            args.push(self.parse_expr());
            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        args
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Number(n) => {
                self.advance();
                Expr::Number(n)
            },
            Token::True => {
                self.advance();
                Expr::BoolLiteral(true)
            },
            Token::False => {
                self.advance();
                Expr::BoolLiteral(false)
            },
            Token::CharLiteral(c) => {
                self.advance();
                Expr::CharLiteral(c)
            },
            Token::StringLiteral(s) => {
                self.advance();
                Expr::StringLiteral(s)
            },
            Token::Identifier(name) => {
                self.advance();
                Expr::Variable(name)
            },
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.eat(Token::RParen);
                expr
            },
            Token::LBrace => self.parse_array_literal(),
            _ => panic!("Unexpected token in expression: {:?}", self.current_token),
        }
    }

    fn parse_array_literal(&mut self) -> Expr {
        self.eat(Token::LBrace);
        let mut elements = Vec::new();
        
        if self.current_token == Token::RBrace {
            self.advance();
            return Expr::ArrayLiteral(elements);
        }
        
        loop {
            elements.push(self.parse_expr());
            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        self.eat(Token::RBrace);
        Expr::ArrayLiteral(elements)
    }
}
