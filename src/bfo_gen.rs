
use crate::ast::{Expr, Stmt, Program, Type};
use crate::token::Token;
use std::collections::HashMap;

pub struct BFOGenerator {
    output: String,
    functions: HashMap<String, Stmt>, // Store function definitions
    arrays: HashMap<String, (usize, usize, Type, Option<Vec<Expr>>)>, // name -> (base_addr, length, element_type, literals)
    variables: HashMap<String, Expr>, // Virtual variables (int, char)
    indent_level: usize,
}

impl BFOGenerator {
    pub fn new() -> Self {
        BFOGenerator {
            output: String::new(),
            functions: HashMap::new(),
            arrays: HashMap::new(),
            variables: HashMap::new(),
            indent_level: 0,
        }
    }

    fn get_array_var_name(&self, name: &str, index: i32) -> String {
        if let Some((_, _, elem_type, _)) = self.arrays.get(name) {
            if *elem_type == Type::Cell {
                return format!("{}_{}", name, index + 1);
            }
        }
        name.to_string()
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
            Stmt::VarDecl { var_type, name, value } => {
                let folded = self.fold_expr(value, &self.variables.clone());
                
                match &var_type {
                    Type::Cell => {
                        // cell is physical
                        match &folded {
                            Expr::Number(_) | Expr::CharLiteral(_) => {
                                self.indent();
                                self.emit(&format!("set {} ", name));
                                self.gen_expr_simple(folded);
                                self.emit_line("");
                            },
                            Expr::Variable(v) => {
                                self.indent();
                                self.emit_line(&format!("set {} 0", name));
                                self.indent();
                                self.emit_line(&format!("add {} {}", name, v));
                            },
                            Expr::BinaryOp { left, op, right } => {
                                self.indent();
                                self.emit_line(&format!("set {} 0", name));
                                
                                // Add left
                                self.indent();
                                self.emit(&format!("add {} ", name));
                                self.gen_expr_simple(*left.clone());
                                self.emit_line("");
                                
                                // Add/Sub right
                                self.indent();
                                let op_cmd = if *op == Token::Plus { "add" } else { "sub" };
                                self.emit(&format!("{} {} ", op_cmd, name));
                                self.gen_expr_simple(*right.clone());
                                self.emit_line("");
                            },
                            _ => {
                                self.indent();
                                self.emit(&format!("set {} ", name));
                                self.gen_expr_simple(folded);
                                self.emit_line("");
                            }
                        }
                    },
                    Type::Array(inner) => {
                        if **inner == Type::Cell {
                            // cell[] is physical: contiguous named cells
                            if let Expr::ArrayLiteral(elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), None));
                                for (i, el) in elements.iter().enumerate() {
                                    let cell_name = format!("{}_{}", name, i + 1);
                                    self.indent();
                                    self.emit(&format!("set {} ", cell_name));
                                    self.gen_expr_simple(el.clone());
                                    self.emit_line("");
                                }
                            }
                        } else {
                            // int[] or char[] is virtual: store in memory
                            if let Expr::StringLiteral(ref s_val) = folded {
                                let char_literals: Vec<Expr> = s_val.chars().map(Expr::CharLiteral).collect();
                                self.arrays.insert(name.clone(), (0, s_val.len(), (**inner).clone(), Some(char_literals)));
                            } else if let Expr::ArrayLiteral(ref elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), Some(elements.clone())));
                            }
                            // Silent: No BFO for virtual arrays
                        }
                    },
                    _ => {
                        // int or char is virtual: store in memory
                        self.variables.insert(name, folded);
                        // Silent: No BFO for virtual variables
                    }
                }
            },
            Stmt::IndexedAssign { name, index, value } => {
                let folded_val = self.fold_expr(value, &self.variables.clone());
                if let Expr::Number(i) = index {
                    if let Some((_, _, elem_type, literals)) = self.arrays.get_mut(&name) {
                        if *elem_type == Type::Cell {
                            // physical cell[] update
                            let cell_name = format!("{}_{}", name, i + 1);
                            self.indent();
                            self.emit(&format!("set {} ", cell_name));
                            self.gen_expr_simple(folded_val);
                            self.emit_line("");
                        } else {
                            // virtual array update (silent)
                            if let Some(lits) = literals {
                                if (i as usize) < lits.len() {
                                    lits[i as usize] = folded_val;
                                }
                            }
                        }
                    }
                }
            },
            Stmt::Assign { name, value } => {
                let folded = self.fold_expr(value, &self.variables.clone());
                // Determine if 'name' is physical or virtual
                // For now, we assume if it was declared as cell it is physical
                // (Need a way to check var_type of existing variables)
                // Actually, let's just check if it's in our virtual 'variables' map
                if self.variables.contains_key(&name) {
                    self.variables.insert(name, folded);
                } else {
                    // Assume physical (cell)
                    match &folded {
                        Expr::Number(_) | Expr::CharLiteral(_) => {
                            self.indent();
                            self.emit(&format!("set {} ", name));
                            self.gen_expr_simple(folded);
                            self.emit_line("");
                        },
                        Expr::Variable(v) => {
                            self.indent();
                            self.emit_line(&format!("set {} 0", name));
                            self.indent();
                            self.emit_line(&format!("add {} {}", name, v));
                        },
                        Expr::BinaryOp { left, op, right } => {
                            self.indent();
                            self.emit_line(&format!("set {} 0", name));
                            
                            // Add left
                            self.indent();
                            self.emit(&format!("add {} ", name));
                            self.gen_expr_simple(*left.clone());
                            self.emit_line("");
                            
                            // Add/Sub right
                            self.indent();
                            let op_cmd = if *op == Token::Plus { "add" } else { "sub" };
                            self.emit(&format!("{} {} ", op_cmd, name));
                            self.gen_expr_simple(*right.clone());
                            self.emit_line("");
                        },
                        _ => {
                            self.indent();
                            self.emit(&format!("set {} ", name));
                            self.gen_expr_simple(folded);
                            self.emit_line("");
                        }
                    }
                }
            },
            Stmt::FuncDecl { name, params, return_type: _, body } => {
                if is_top_level {
                    if self.is_predictable_function(&params) {
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
                        self.emit_line(&format!("; unpredictable function {}", name));
                    }
                }
            },
            Stmt::Putc(expr) => {
                let folded = self.fold_expr(expr, &self.variables.clone());
                
                self.indent();
                match folded {
                    Expr::Variable(name) => {
                        // If it's a virtual array, print sequentially
                        let array_data = if let Some((_, _, elem_type, literals)) = self.arrays.get(&name) {
                            if *elem_type == Type::Char {
                                literals.clone().map(|lits| (name.clone(), lits))
                            } else { None }
                        } else { None };

                        if let Some((name, lits)) = array_data {
                            for (i, lit) in lits.iter().enumerate() {
                                if i > 0 { self.indent(); }
                                self.emit(&format!("set {} ", name));
                                self.gen_expr_simple(lit.clone());
                                self.emit_line("");
                                self.indent();
                                self.emit_line(&format!("print {}", name));
                            }
                        } else {
                            // Single physical variable or unknown
                            self.emit_line(&format!("print {}", name));
                        }
                    },
                    Expr::Number(n) => {
                        self.emit_line(&format!("print {}", n));
                    },
                    Expr::CharLiteral(c) => {
                        self.emit_line(&format!("print '{}'", c.escape_default()));
                    },
                    Expr::StringLiteral(s) => {
                        for ch in s.chars() {
                            self.indent();
                            self.emit_line(&format!("print '{}'", ch.escape_default()));
                        }
                    },
                    Expr::ArrayLiteral(elements) => {
                        for el in elements {
                            self.indent();
                            self.emit("print ");
                            self.gen_expr_simple(el.clone());
                            self.emit_line("");
                        }
                    },
                    _ => {
                        // If it wasn't a literal/variable after folding, try one last resolution
                        // This handles cases like `putc(a + b)` where a and b were globals
                        self.emit("print ");
                        self.gen_expr_simple(folded);
                        self.emit_line("");
                    }
                }
            },
            Stmt::For { init, condition, update, body } => {
                // Check if we can unroll the loop (constant iteration count)
                let can_optimize = self.can_optimize_for_loop(&init, &condition, &update);
                
                if let Some((var_name, limit)) = can_optimize {
                    // Check if this loop is iterating over a streaming array (non-cell)
                    // Pattern: for (int i=0; i < arr.length; i++) { ... arr[i] ... }
                    // If arr is streaming, we need to set the value in each iteration
                    

                    // Loop unrolling: repeat the body N times
                    for i in 0..limit {
                        for s in &body {
                            // Substitute loop variable with current index
                            let substituted_s = self.substitute_stmt(s.clone(), &var_name, i);
                            self.gen_stmt(substituted_s, false);
                        }
                    }
                } else {
                    // Fallback: Standard for loop conversion to while
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
            Stmt::Forn { name, count, body } => {
                let folded_count = self.fold_expr(count, &self.variables.clone());
                // Generate native countdown loop: set n value; while n { body; sub n 1 }
                self.indent();
                // Special optimization: if count is a variable and we are in a function,
                // we can often use it directly if we handle the BFO 'set' restriction.
                match &folded_count {
                    Expr::Number(n) => self.emit_line(&format!("set {} {}", name, n)),
                    Expr::CharLiteral(c) => self.emit_line(&format!("set {} '{}'", name, c.escape_default())),
                    Expr::Variable(v) => {
                        // Use the 'add-to-zero' trick to match user's efficient copying
                        self.emit_line(&format!("set {} 0", name));
                        self.indent();
                        self.emit_line(&format!("add {} {}", name, v));
                    },
                    _ => {
                        self.emit(&format!("set {} ", name));
                        self.gen_expr_simple(folded_count);
                        self.emit_line("");
                    }
                }
                
                self.indent();
                self.emit_line(&format!("while {} {{", name));
                
                self.indent_level += 1;
                for s in body {
                    self.gen_stmt(s, false);
                }
                
                // Decrement counter
                self.indent();
                self.emit_line(&format!("sub {} 1", name));
                self.indent_level -= 1;
                
                self.indent();
                self.emit_line("}");
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
                if let Expr::FunctionCall { name, args } = expr {
                    if let Some(func_def) = self.functions.get(&name).cloned() {
                        if let Stmt::FuncDecl { params, body, .. } = func_def {
                            if !self.is_predictable_function(&params) {
                                self.inline_function(params, args, body);
                                return;
                            }
                        }
                    }
                    
                    self.indent();
                    self.emit(&format!("{}(", name));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { self.emit(", "); }
                        self.gen_expr(arg.clone());
                    }
                    self.emit_line(")");
                }
            },
        }
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' { self.emit("'\\n'"); }
                else if c == '\t' { self.emit("'\\t'"); }
                else { self.emit(&format!("'{}'", c)); }
            },
            Expr::Variable(name) => {
                if let Some(val) = self.variables.get(&name) {
                    // Recursively resolve in case of nested virtuals
                    self.gen_expr(val.clone());
                } else {
                    self.emit(&name);
                }
            },
            Expr::FunctionCall { name, args } => {
                self.emit(&format!("{}(", name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_expr(arg.clone());
                }
                self.emit(")");
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = &*index {
                    match &*array {
                        Expr::Variable(name) => {
                            // If it's a streaming array, we can return its literal value if known
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if *elem_type != Type::Cell {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        self.gen_expr(lit.clone());
                                        return;
                                    }
                                }
                            }
                            self.emit(&self.get_array_var_name(name, *i));
                        },
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                if ch == '\n' { self.emit("'\\n'"); }
                                else if ch == '\t' { self.emit("'\\t'"); }
                                else { self.emit(&format!("'{}'", ch)); }
                            } else { panic!("String index out of bounds: {} in {:?}", i, s); }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                self.gen_expr(el.clone());
                            } else { panic!("Array index out of bounds: {} in {:?}", i, elements); }
                        },
                        _ => panic!("Complex array indexing not supported"),
                    }
                } else {
                    panic!("Only constant array indexing supported");
                }
            },
            Expr::BinaryOp { left, op, right } => {
                self.emit("(");
                self.gen_expr(*left);
                self.emit(&format!(" {} ", match op {
                    Token::Plus => "+",
                    Token::Minus => "-",
                    _ => "?",
                }));
                self.gen_expr(*right);
                self.emit(")");
            },
            Expr::MemberAccess { object, member } => {
                if let Expr::Variable(name) = *object {
                    if member == "length" {
                        if let Some((_, len, _, _)) = self.arrays.get(&name) {
                            self.emit(&len.to_string());
                        } else { panic!("Undefined array: {}", name); }
                    } else { panic!("Unknown member: {}", member); }
                } else { panic!("Member access only supported on variables"); }
            },
            _ => panic!("Complex expression not supported in this context"),
        }
    }

    fn gen_expr_simple(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.emit(&n.to_string()),
            Expr::CharLiteral(c) => {
                if c == '\n' { self.emit("'\\n'"); }
                else if c == '\t' { self.emit("'\\t'"); }
                else { self.emit(&format!("'{}'", c)); }
            },
            Expr::Variable(name) => {
                if let Some(val) = self.variables.get(&name) {
                    // Recursively resolve to literal
                    self.gen_expr_simple(val.clone());
                } else {
                    // Assume literal or physical name
                    self.emit(&name);
                }
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = &*index {
                    match &*array {
                        Expr::Variable(name) => {
                            // If it's a streaming array, we can return its literal value if known
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if *elem_type != Type::Cell {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        self.gen_expr_simple(lit.clone());
                                        return;
                                    }
                                }
                            }
                            // Otherwise, we can't 'set' from a cell array directly in BFO
                            panic!("Cannot 'set' from cell array element directly in BFO: {}_{}. BFO only supports set <var> <literal>.", name, i + 1);
                        },
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                if ch == '\n' { self.emit("'\\n'"); }
                                else if ch == '\t' { self.emit("'\\t'"); }
                                else { self.emit(&format!("'{}'", ch)); }
                            } else { panic!("String index out of bounds: {} in {:?}", i, s); }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                self.gen_expr_simple(el.clone());
                            } else { panic!("Array index out of bounds: {} in {:?}", i, elements); }
                        },
                        _ => panic!("Complex array indexing not supported in 'set' context."),
                    }
                } else {
                    panic!("Only constant array indexing supported in 'set' context.");
                }
            },
            Expr::MemberAccess { object, member } => {
                if let Expr::Variable(name) = *object {
                    if member == "length" {
                        if let Some((_, len, _, _)) = self.arrays.get(&name) {
                            self.emit(&len.to_string());
                        } else { panic!("Undefined array: {}", name); }
                    } else { panic!("Unknown member: {}", member); }
                } else { panic!("Member access only supported on variables"); }
            },
            _ => panic!("Only simple expressions (literals) allowed in BFO 'set' context: {:?}", expr),
        }
    }

    fn optimize_program(&mut self, program: Program) -> Program {
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
                    
                    // If this is a virtual scalar variable with a constant value, just track it
                    if var_type == Type::Int || var_type == Type::Char {
                        match &folded_value {
                            Expr::Number(_) | Expr::CharLiteral(_) => {
                                var_values.insert(name.clone(), folded_value.clone());
                                self.variables.insert(name.clone(), folded_value);
                                // Don't emit this variable yet - it's virtual and silent
                                continue;
                            },
                            _ => {}
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
                
                // Helper to get numeric value of Number or CharLiteral
                let to_num = |e: &Expr| match e {
                    Expr::Number(n) => Some(*n),
                    Expr::CharLiteral(c) => Some(*c as i32),
                    _ => None,
                };

                // Try to evaluate constant expressions
                if let (Some(l), Some(r)) = (to_num(&left_folded), to_num(&right_folded)) {
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
            _ => expr, // Other expressions (CharLiteral, StringLiteral, Number) unchanged
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
        
        // Check condition: i < n OR i < arr.length
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
            
            match right.as_ref() {
                Expr::Number(n) => *n,
                Expr::MemberAccess { object, member } => {
                    if member == "length" {
                        match object.as_ref() {
                            Expr::Variable(array_name) => {
                                if let Some((_, len, _, _)) = self.arrays.get(array_name) {
                                    *len as i32
                                } else {
                                    return None;
                                }
                            },
                            Expr::StringLiteral(s) => s.len() as i32,
                            Expr::ArrayLiteral(elements) => elements.len() as i32,
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                },
                _ => return None,
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
        // Functions with array parameters are not predictable because they 
        // require unrolling. int/char/cell parameters are fine in BFO functions.
        for (param_type, _) in params {
            if let Type::Array(_) = param_type {
                return false;
            }
        }
        true
    }

    fn inline_function(&mut self, params: Vec<(Type, String)>, args: Vec<Expr>, body: Vec<Stmt>) {
        let mut inlined_body = body;
        
        // Substitute each parameter with its argument
        for (i, (_, param_name)) in params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                // If the argument is a literal, we can do direct substitution
                // For now, let's treat variables and literals similarly in substitution
                let mut new_body = Vec::new();
                for stmt in inlined_body {
                    new_body.push(self.substitute_stmt_with_expr(stmt, param_name, arg.clone()));
                }
                inlined_body = new_body;
            }
        }
        
        // Generate code for the substituted body
        for stmt in inlined_body {
            self.gen_stmt(stmt, false);
        }
    }

    fn substitute_stmt_with_expr(&self, stmt: Stmt, var_name: &str, replacement: Expr) -> Stmt {
        match stmt {
            Stmt::VarDecl { var_type, name, value: expr } => Stmt::VarDecl {
                var_type,
                name,
                value: self.substitute_expr_with_expr(expr, var_name, replacement),
            },
            Stmt::Assign { name, value: expr } => Stmt::Assign {
                name,
                value: self.substitute_expr_with_expr(expr, var_name, replacement),
            },
            Stmt::IndexedAssign { name, index, value: expr } => Stmt::IndexedAssign {
                name,
                index: self.substitute_expr_with_expr(index, var_name, replacement.clone()),
                value: self.substitute_expr_with_expr(expr, var_name, replacement),
            },
            Stmt::Putc(expr) => Stmt::Putc(self.substitute_expr_with_expr(expr,var_name, replacement)),
            Stmt::Forn { name, count, body } => Stmt::Forn {
                name,
                count: self.substitute_expr_with_expr(count, var_name, replacement.clone()),
                body: body.into_iter().map(|s| self.substitute_stmt_with_expr(s, var_name, replacement.clone())).collect(),
            },
            Stmt::While { condition, body } => Stmt::While {
                condition: self.substitute_expr_with_expr(condition, var_name, replacement.clone()),
                body: body.into_iter().map(|s| self.substitute_stmt_with_expr(s, var_name, replacement.clone())).collect(),
            },
            Stmt::ExprStmt(expr) => Stmt::ExprStmt(self.substitute_expr_with_expr(expr, var_name, replacement)),
            _ => stmt,
        }
    }

    fn substitute_expr_with_expr(&self, expr: Expr, var_name: &str, replacement: Expr) -> Expr {
        match expr {
            Expr::Variable(name) => {
                if name == var_name {
                    replacement
                } else {
                    Expr::Variable(name)
                }
            },
            Expr::StringLiteral(_) | Expr::CharLiteral(_) | Expr::Number(_) => expr,
            Expr::BinaryOp { left, op, right } => Expr::BinaryOp {
                left: Box::new(self.substitute_expr_with_expr(*left, var_name, replacement.clone())),
                op,
                right: Box::new(self.substitute_expr_with_expr(*right, var_name, replacement)),
            },
            Expr::ArrayAccess { array, index } => Expr::ArrayAccess {
                array: Box::new(self.substitute_expr_with_expr(*array, var_name, replacement.clone())),
                index: Box::new(self.substitute_expr_with_expr(*index, var_name, replacement)),
            },
            Expr::MemberAccess { object, member } => Expr::MemberAccess {
                object: Box::new(self.substitute_expr_with_expr(*object, var_name, replacement)),
                member,
            },
            Expr::FunctionCall { name, args } => Expr::FunctionCall {
                name,
                args: args.into_iter().map(|a| self.substitute_expr_with_expr(a, var_name, replacement.clone())).collect(),
            },
            _ => expr,
        }
    }
    fn substitute_stmt(&self, stmt: Stmt, var_name: &str, value: i32) -> Stmt {
        match stmt {
            Stmt::VarDecl { var_type, name, value: expr } => Stmt::VarDecl {
                var_type,
                name,
                value: self.substitute_expr(expr, var_name, value),
            },
            Stmt::Assign { name, value: expr } => Stmt::Assign {
                name,
                value: self.substitute_expr(expr, var_name, value),
            },
            Stmt::IndexedAssign { name, index, value: expr } => Stmt::IndexedAssign {
                name,
                index: self.substitute_expr(index, var_name, value),
                value: self.substitute_expr(expr, var_name, value),
            },
            Stmt::Putc(expr) => Stmt::Putc(self.substitute_expr(expr, var_name, value)),
            Stmt::Forn { name, count, body } => Stmt::Forn {
                name,
                count: self.substitute_expr(count, var_name, value),
                body: body.into_iter().map(|s| self.substitute_stmt(s, var_name, value)).collect(),
            },
            Stmt::While { condition, body } => Stmt::While {
                condition: self.substitute_expr(condition, var_name, value),
                body: body.into_iter().map(|s| self.substitute_stmt(s, var_name, value)).collect(),
            },
            Stmt::ExprStmt(expr) => Stmt::ExprStmt(self.substitute_expr(expr, var_name, value)),
            _ => stmt,
        }
    }

    fn substitute_expr(&self, expr: Expr, var_name: &str, value: i32) -> Expr {
        match expr {
            Expr::Variable(name) if name == var_name => Expr::Number(value),
            Expr::CharLiteral(_) | Expr::StringLiteral(_) | Expr::Number(_) => expr,
            Expr::BinaryOp { left, op, right } => Expr::BinaryOp {
                left: Box::new(self.substitute_expr(*left, var_name, value)),
                op,
                right: Box::new(self.substitute_expr(*right, var_name, value)),
            },
            Expr::ArrayAccess { array, index } => Expr::ArrayAccess {
                array: Box::new(self.substitute_expr(*array, var_name, value)),
                index: Box::new(self.substitute_expr(*index, var_name, value)),
            },
            Expr::MemberAccess { object, member } => Expr::MemberAccess {
                object: Box::new(self.substitute_expr(*object, var_name, value)),
                member,
            },
            Expr::FunctionCall { name, args } => Expr::FunctionCall {
                name,
                args: args.into_iter().map(|a| self.substitute_expr(a, var_name, value)).collect(),
            },
            _ => expr,
        }
    }
}
