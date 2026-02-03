
use crate::ast::{Expr, Stmt, Program, Type};
use crate::token::Token;
use std::collections::HashMap;

pub struct BFOGenerator {
    output: String,
    functions: HashMap<String, Stmt>, // Store function definitions
    global_counter: usize,
    indent_level: usize,
}

impl BFOGenerator {
    pub fn new() -> Self {
        BFOGenerator {
            output: String::new(),
            functions: HashMap::new(),
            global_counter: 0,
            indent_level: 0,
        }
    }

    pub fn generate(&mut self, program: Program) -> String {
        // First pass: collect function definitions
        for stmt in &program.statements {
            if let Stmt::FuncDecl { name, .. } = stmt {
                self.functions.insert(name.clone(), stmt.clone());
            }
        }

        // Optimization pass: fold constants and eliminate dead variables
        let optimized_program = self.optimize_program(program);

        // Second pass: generate code
        for stmt in optimized_program.statements {
            self.gen_stmt(stmt, true);
        }

        self.output.clone()
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn gen_stmt(&mut self, stmt: Stmt, is_top_level: bool) {
        match stmt {
            Stmt::VarDecl { var_type: _, name, value } => {
                // Handle different expression types
                match value {
                    Expr::BinaryOp { left, op, right } => {
                        // set name left_value
                        // add/sub name right_value
                        self.indent();
                        self.emit(&format!("set {} ", name));
                        self.gen_expr_simple(*left);
                        self.emit_line("");
                        
                        match op {
                            Token::Plus => {
                                self.indent();
                                self.emit(&format!("add {} ", name));
                                self.gen_expr_simple(*right);
                                self.emit_line("");
                            },
                            Token::Minus => {
                                self.indent();
                                self.emit(&format!("sub {} ", name));
                                self.gen_expr_simple(*right);
                                self.emit_line("");
                            },
                            _ => panic!("Unsupported binary operator in BFO: {:?}", op),
                        }
                    },
                    _ => {
                        // Simple assignment
                        self.indent();
                        self.emit(&format!("set {} ", name));
                        self.gen_expr_simple(value);
                        self.emit_line("");
                    }
                }
            },
            Stmt::Assign { name, value } => {
                match value {
                    Expr::BinaryOp { left, op, right } => {
                        self.indent();
                        self.emit(&format!("set {} ", name));
                        self.gen_expr_simple(*left);
                        self.emit_line("");
                        
                        match op {
                            Token::Plus => {
                                self.indent();
                                self.emit(&format!("add {} ", name));
                                self.gen_expr_simple(*right);
                                self.emit_line("");
                            },
                            Token::Minus => {
                                self.indent();
                                self.emit(&format!("sub {} ", name));
                                self.gen_expr_simple(*right);
                                self.emit_line("");
                            },
                            _ => panic!("Unsupported binary operator"),
                        }
                    },
                    _ => {
                        self.indent();
                        self.emit(&format!("set {} ", name));
                        self.gen_expr_simple(value);
                        self.emit_line("");
                    }
                }
            },
            Stmt::FuncDecl { name, params, return_type: _, body } => {
                if is_top_level {
                    // Check if function is predictable
                    if self.is_predictable_function(&params) {
                        // Emit function definition
                        self.emit(&format!("func {}(", name));
                        for (i, (_, param_name)) in params.iter().enumerate() {
                            if i > 0 { self.emit(", "); }
                            self.emit(param_name);
                        }
                        self.emit_line(") {");
                        
                        self.indent_level += 1;
                        for s in body {
                            self.gen_stmt(s, false);
                        }
                        self.indent_level -= 1;
                        
                        self.emit_line("}");
                    } else {
                        // Unpredictable function - will be inlined at call sites
                        self.emit_line(&format!("; function {} breaks down due to not using string in parameter", name));
                        self.emit_line("; such functions are not predictable and be decoded in this object file");
                    }
                }
            },
            Stmt::Putc(expr) => {
                self.indent();
                self.emit("print ");
                self.gen_expr(expr);
                self.emit_line("");
            },
            Stmt::For { init, condition, update, body } => {
                // Optimization: Detect for (int i = 0; i < n; i++) pattern
                // Transform to: i = 0; sub i n; while i { body; add i 1 }
                // This eliminates the need for comparison operations
                
                let can_optimize = self.can_optimize_for_loop(&init, &condition, &update);
                
                if let Some((var_name, limit)) = can_optimize {
                    // Optimized countdown pattern
                    self.indent();
                    self.emit_line(&format!("set {} 0", var_name));
                    self.indent();
                    self.emit_line(&format!("sub {} {}", var_name, limit));
                    
                    self.indent();
                    self.emit_line(&format!("while {} {{", var_name));
                    
                    self.indent_level += 1;
                    for s in body {
                        self.gen_stmt(s, false);
                    }
                    
                    // Increment (counting toward 0)
                    self.indent();
                    self.emit_line(&format!("add {} 1", var_name));
                    self.indent_level -= 1;
                    
                    self.indent();
                    self.emit_line("}");
                } else {
                    // Fallback: Standard for loop conversion
                    // init
                    self.gen_stmt(*init, false);
                    
                    // while condition
                    self.indent();
                    self.emit("while ");
                    match &condition {
                        Expr::Variable(name) => self.emit(name),
                        Expr::BinaryOp { left, op: _, right: _ } => {
                            if let Expr::Variable(name) = left.as_ref() {
                                self.emit(name);
                            } else {
                                panic!("Complex comparison not supported");
                            }
                        },
                        _ => panic!("Unsupported for loop condition"),
                    }
                    self.emit_line(" {");
                    
                    self.indent_level += 1;
                    for s in body {
                        self.gen_stmt(s, false);
                    }
                    
                    // update
                    self.gen_stmt(*update, false);
                    self.indent_level -= 1;
                    
                    self.indent();
                    self.emit_line("}");
                }
            },
            Stmt::While { condition, body } => {
                self.indent();
                self.emit("while ");
                // For while loops, we only support simple variable conditions in BFO
                // Comparison expressions like i < n need to be handled differently
                // For now, we'll just emit the left side of comparison (simplified)
                match &condition {
                    Expr::Variable(name) => self.emit(name),
                    Expr::BinaryOp { left, op: _, right: _ } => {
                        // For comparisons, we need a temp variable
                        // This is a simplification - proper implementation would need
                        // to compute the comparison result
                        if let Expr::Variable(name) = left.as_ref() {
                            self.emit(name);
                        } else {
                            panic!("Complex comparison not supported");
                        }
                    },
                    _ => panic!("Unsupported while condition"),
                }
                self.emit_line(" {");
                
                self.indent_level += 1;
                for s in body {
                    self.gen_stmt(s, false);
                }
                self.indent_level -= 1;
                
                self.indent();
                self.emit_line("}");
            },
            Stmt::ExprStmt(expr) => {
                // Function call
                if let Expr::FunctionCall { name, args } = expr {
                    // Check if we should inline
                    if let Some(func_def) = self.functions.get(&name).cloned() {
                        if let Stmt::FuncDecl { params, body, .. } = func_def {
                            if !self.is_predictable_function(&params) {
                                // Inline the function
                                self.inline_function(params, args, body);
                                return;
                            }
                        }
                    }
                    
                    // Normal function call
                    self.indent();
                    self.emit(&format!("{}(", name));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { self.emit(", "); }
                        self.gen_expr(arg.clone());
                    }
                    self.emit_line(")");
                } else {
                    panic!("ExprStmt must be a function call");
                }
            },
        }
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' {
                    self.emit("'\\n'");
                } else if c == '\t' {
                    self.emit("'\\t'");
                } else {
                    self.emit(&format!("'{}'", c));
                }
            },
            Expr::Variable(name) => self.emit(&name),
            Expr::FunctionCall { name, args } => {
                self.emit(&format!("{}(", name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_expr(arg.clone());
                }
                self.emit(")");
            },
            _ => panic!("Complex expression not supported in this context"),
        }
    }

    fn gen_expr_simple(&mut self, expr: Expr) {
        // For simple value context (right side of set/add/sub)
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' {
                    self.emit("'\\n'");
                } else if c == '\t' {
                    self.emit("'\\t'");
                } else {
                    self.emit(&format!("'{}'", c));
                }
            },
            Expr::Variable(name) => self.emit(&name),
            _ => panic!("Only simple expressions allowed in value context"),
        }
    }

    fn optimize_program(&self, program: Program) -> Program {
        // Optimization: Fold int variables into cell variables when possible
        // Since only cell can be used for I/O, int variables are just intermediate
        
        let mut optimized_stmts = Vec::new();
        let mut var_values: HashMap<String, Expr> = HashMap::new();
        let mut var_types: HashMap<String, Type> = HashMap::new();
        
        for stmt in program.statements {
            match stmt {
                Stmt::VarDecl { var_type, name, value } => {
                    // Track variable values for folding
                    var_types.insert(name.clone(), var_type.clone());
                    
                    // Try to evaluate the expression with known values
                    let folded_value = self.fold_expr(value, &var_values);
                    
                    // If this is an int variable with a constant value, just track it
                    if var_type == Type::Int {
                        if let Expr::Number(_) = folded_value {
                            var_values.insert(name.clone(), folded_value);
                            // Don't emit this variable yet - might be folded later
                            continue;
                        }
                    }
                    
                    // If this is a cell variable, fold any int dependencies
                    if var_type == Type::Cell {
                        let final_value = self.fold_expr(folded_value, &var_values);
                        optimized_stmts.push(Stmt::VarDecl {
                            var_type,
                            name: name.clone(),
                            value: final_value,
                        });
                        var_values.insert(name.clone(), Expr::Variable(name));
                        continue;
                    }
                    
                    // Otherwise keep the statement
                    var_values.insert(name.clone(), folded_value.clone());
                    optimized_stmts.push(Stmt::VarDecl {
                        var_type,
                        name,
                        value: folded_value,
                    });
                },
                Stmt::FuncDecl { .. } => {
                    // Functions are kept as-is
                    optimized_stmts.push(stmt);
                },
                _ => {
                    // Other statements kept as-is
                    optimized_stmts.push(stmt);
                }
            }
        }
        
        Program { statements: optimized_stmts }
    }

    fn fold_expr(&self, expr: Expr, var_values: &HashMap<String, Expr>) -> Expr {
        match expr {
            Expr::Variable(name) => {
                // Substitute variable with its value if known
                if let Some(value) = var_values.get(&name) {
                    value.clone()
                } else {
                    Expr::Variable(name)
                }
            },
            Expr::BinaryOp { left, op, right } => {
                let left_folded = self.fold_expr(*left, var_values);
                let right_folded = self.fold_expr(*right, var_values);
                
                // Try to evaluate constant expressions
                if let (Expr::Number(l), Expr::Number(r)) = (&left_folded, &right_folded) {
                    match op {
                        Token::Plus => return Expr::Number(l + r),
                        Token::Minus => return Expr::Number(l - r),
                        _ => {}
                    }
                }
                
                // Can't fold, return the folded operands
                Expr::BinaryOp {
                    left: Box::new(left_folded),
                    op,
                    right: Box::new(right_folded),
                }
            },
            _ => expr, // Other expressions unchanged
        }
    }

    fn can_optimize_for_loop(&self, init: &Stmt, condition: &Expr, update: &Stmt) -> Option<(String, i32)> {
        // Pattern: for (int i = 0; i < n; i++)
        // Returns: Some((var_name, n)) if pattern matches
        
        // Check init: int i = 0
        let var_name = if let Stmt::VarDecl { var_type: _, name, value } = init {
            if let Expr::Number(0) = value {
                name.clone()
            } else {
                return None;
            }
        } else {
            return None;
        };
        
        // Check condition: i < n
        let limit = if let Expr::BinaryOp { left, op, right } = condition {
            if *op != Token::Less {
                return None;
            }
            if let Expr::Variable(cond_var) = left.as_ref() {
                if cond_var != &var_name {
                    return None;
                }
            } else {
                return None;
            }
            if let Expr::Number(n) = right.as_ref() {
                *n
            } else {
                return None;
            }
        } else {
            return None;
        };
        
        // Check update: i++ (which is i = i + 1)
        if let Stmt::Assign { name: update_var, value } = update {
            if update_var != &var_name {
                return None;
            }
            if let Expr::BinaryOp { left, op, right } = value {
                if *op != Token::Plus {
                    return None;
                }
                if let Expr::Variable(left_var) = left.as_ref() {
                    if left_var != &var_name {
                        return None;
                    }
                } else {
                    return None;
                }
                if let Expr::Number(1) = right.as_ref() {
                    // Pattern matches!
                    return Some((var_name, limit));
                }
            }
        }
        
        None
    }

    fn is_predictable_function(&self, params: &[(Type, String)]) -> bool {
        // A function is predictable if it doesn't have string parameters
        for (param_type, _) in params {
            if *param_type == Type::String {
                return false;
            }
        }
        true
    }

    fn inline_function(&mut self, params: Vec<(Type, String)>, args: Vec<Expr>, body: Vec<Stmt>) {
        // For string functions, we need to inline completely
        // This is complex - for now, handle the print_string case
        
        // Check if this is a string printing function
        if params.len() == 1 && params[0].0 == Type::String {
            if let Expr::StringLiteral(s) = &args[0] {
                // Inline string printing
                for ch in s.chars() {
                    let temp_var = format!("g{}", self.global_counter);
                    self.global_counter += 1;
                    
                    self.emit_line(&format!("set {} '{}'", temp_var, 
                        if ch == '\n' { "\\n".to_string() } 
                        else if ch == '\t' { "\\t".to_string() }
                        else { ch.to_string() }));
                    self.emit_line(&format!("print {}", temp_var));
                }
            }
        }
    }
}
