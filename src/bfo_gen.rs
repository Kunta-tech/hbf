
use crate::hbf_ast::{Expr, Stmt, Program, Type};
use crate::hbf_token::Token;
use std::collections::HashMap;


pub struct BFOGenerator {
    output: String,
    functions: HashMap<String, Stmt>, // Store function definitions
    arrays: HashMap<String, (usize, usize, Type, Option<Vec<Expr>>)>, // name -> (base_addr, length, element_type, literals)
    variables: Vec<HashMap<String, Expr>>, // Scoped virtual variables (int, char)
    indent_level: usize,
    forn_counter: usize,
    native_loop_depth: usize,
}

impl BFOGenerator {
    pub fn new() -> Self {
        BFOGenerator {
            output: String::new(),
            functions: HashMap::new(),
            arrays: HashMap::new(),
            variables: vec![HashMap::new()],
            indent_level: 0,
            forn_counter: 0,
            native_loop_depth: 0,
        }
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.variables.len() > 1 {
            self.variables.pop();
        }
    }

    fn get_variable(&self, name: &str) -> Option<Expr> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn set_variable(&mut self, name: &str, val: Expr) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), val);
        }
    }

    fn get_array_var_name(&self, name: &str, index: i32) -> String {
        if let Some((_, _, elem_type, _)) = self.arrays.get(name) {
            if !elem_type.is_virtual() {
                return format!("__hbf_cell_{}_{}", name, index);
            }
        }
        name.to_string()
    }
    pub fn generate(&mut self, program: Program) -> String {
        // Collect function definitions
        for stmt in &program.statements {
            if let Stmt::FuncDecl { name, .. } = stmt {
                self.functions.insert(name.clone(), stmt.clone());
            }
        }

        self.push_scope();

        // Generate code (simulation/inlining happens here)
        for stmt in program.statements {
            self.gen_stmt(stmt, true);
        }

        self.pop_scope();

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

    fn emit_set(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("set {} {}", name, val));
    }
    fn emit_new(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("new {} {}", name, val));
    }

    fn emit_add(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("add {} {}", name, val));
    }

    fn emit_sub(&mut self, name: &str, val: &str) {
        self.indent();
        self.emit_line(&format!("sub {} {}", name, val));
    }

    fn materialize_to_cell(&mut self, name: &str, expr: Expr, is_new: bool) {
        match &expr {
            Expr::Number(n) => {
                if is_new {
                    self.emit_new(name, &n.to_string());
                } else {
                    self.emit_set(name, &n.to_string());
                }
            }
            Expr::CharLiteral(c) => {
                if is_new {
                    self.emit_new(name, &format!("'{}'", c.escape_default()));
                } else {
                    self.emit_set(name, &format!("'{}'", c.escape_default()));
                }
            }
            Expr::BoolLiteral(b) => {
                if is_new {
                    self.emit_new(name, if *b { "1" } else { "0" });
                } else {
                    self.emit_set(name, if *b { "1" } else { "0" });
                }
            }
            Expr::Variable(v) => {
                if name == v { return; }
                if is_new {
                    self.emit_new(name, "0");
                } else {
                    self.emit_set(name, "0");
                }
                self.emit_add(name, v);
            },
            Expr::ArrayAccess { array, index } => {
                // If it's a physical array access, we treat it as a variable for the copy
                if let Expr::Number(i) = index.as_ref() {
                    if let Expr::Variable(array_name) = array.as_ref() {
                        if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                            if !elem_type.is_virtual() {
                                let cell_name = self.get_array_var_name(array_name, *i);
                                self.emit_add(name, &cell_name);
                                return;
                            }
                        }
                    }
                }
                
                // Otherwise fall back to gen_expr_simple
                self.indent();
                if is_new {
                    self.emit(&format!("new {} ", name));
                } else {
                    self.emit(&format!("set {} ", name));
                }
                self.gen_expr_simple(expr);
                self.emit_line("");
            },
            Expr::BinaryOp { left, op, right } => {
                // Check for shorthands: A = A + B, A = B + A, A = A - B
                let left_is_name = if let Expr::Variable(v) = left.as_ref() { v == name } else { false };
                let right_is_name = if let Expr::Variable(v) = right.as_ref() { v == name } else { false };

                if left_is_name && *op == Token::Plus {
                    // A = A + right  =>  add A right
                    self.indent();
                    self.emit(&format!("add {} ", name));
                    self.gen_expr_simple(*right.clone());
                    self.emit_line("");
                } else if right_is_name && *op == Token::Plus {
                    // A = left + A  =>  add A left
                    self.indent();
                    self.emit(&format!("add {} ", name));
                    self.gen_expr_simple(*left.clone());
                    self.emit_line("");
                } else if left_is_name && *op == Token::Minus {
                    // A = A - right  =>  sub A right
                    self.indent();
                    self.emit(&format!("sub {} ", name));
                    self.gen_expr_simple(*right.clone());
                    self.emit_line("");
                } else {
                    // General case: clear and rebuild
                    self.emit_new(name, "0");
                    
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
                }
            },
            _ => {
                self.indent();
                if is_new {
                    self.emit(&format!("new {} ", name));
                } else {
                    self.emit(&format!("set {} ", name));
                }
                self.gen_expr_simple(expr);
                self.emit_line("");
            }
        }
    }
    fn free_cell(&mut self, name: &str) {
        self.indent();
        self.emit_line(&format!("free {}", name));
    }

    fn gen_stmt(&mut self, stmt: Stmt, is_top_level: bool) {
        match stmt {
            Stmt::VarDecl { var_type, name, value } => {
                let folded = self.fold_expr(value);
    
                match &var_type {
                    Type::Cell => self.materialize_to_cell(&name, folded, true),
                    Type::Array(inner) => {
                        if !inner.is_virtual() {
                            // cell[] is physical: contiguous named cells
                            if let Expr::ArrayLiteral(elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), None));
                                for (i, el) in elements.iter().enumerate() {
                                    let cell_name = format!("{}_{}", name, i);
                                    // Use original name pattern, or maybe allow collisions for array elements?
                                    // Actually HBF arrays are flat. The original code used to do:
                                    // let cell_name = self.get_array_var_name(&name, i);
                                    // Let's stick to a simple consistent naming.
                                    // Reverting to previous state:
                                    let cell_name = self.get_array_var_name(&name, i as i32);
                                    self.materialize_to_cell(&cell_name, el.clone(), true);
                                }
                            } else {
                                // Default array init? Usually arrays are sized.
                                self.arrays.insert(name, (0, 0, (**inner).clone(), None));
                            }
                        } else {
                            // int[] or char[] or bool[] is virtual: store in memory
                            if let Expr::StringLiteral(ref s_val) = folded {
                                let char_literals: Vec<Expr> = s_val.chars().map(Expr::CharLiteral).collect();
                                self.arrays.insert(name.clone(), (0, s_val.len(), (**inner).clone(), Some(char_literals)));
                            } else if let Expr::ArrayLiteral(ref elements) = folded {
                                self.arrays.insert(name.clone(), (0, elements.len(), (**inner).clone(), Some(elements.clone())));
                            }
                        }
                    },
                    _ => {
                        // int or char is virtual: store in memory (current scope)
                        self.set_variable(&name, folded);
                    }
                }
            },
            Stmt::IndexedAssign { name, index, value } => {
                let folded_val = self.fold_expr(value);
                let folded_index = self.fold_expr(index);
                if let Expr::Number(i) = folded_index {
                    if let Some((_, _, elem_type, literals)) = self.arrays.get_mut(&name) {
                        if !elem_type.is_virtual() {
                            // physical cell[] update
                            let cell_name = self.get_array_var_name(&name, i);
                            self.materialize_to_cell(&cell_name, folded_val, false);
                        } else {
                            // virtual array update (silent)
                            if self.native_loop_depth > 0 {
                                eprintln!("WARNING: Modifying virtual array '{}' inside a native loop. This value will not update in the generated BFO.", name);
                            }
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
                let folded = self.fold_expr(value);
                // println!("Assigning {:?} to {:?}", folded, name);
                // Determine if 'name' is physical or virtual
                if self.get_variable(&name).is_some() {
                    if self.native_loop_depth > 0 {
                        eprintln!("WARNING: Modifying virtual variable '{}' inside a native loop. This value will not update in the generated BFO.", name);
                    }
                    self.set_variable(&name, folded);
                } else {
                    self.materialize_to_cell(&name, folded, false);
                }
            },
            Stmt::FuncDecl { name, params: _, return_type: _, body: _ } => {
                if is_top_level {
                     // We already collected functions in the first pass.
                     // In simulation-only mode, we do NOT emit the function definition to BFO.
                     // Debug/comment line:
                     self.emit_line(&format!("; defined function {}", name));
                }
            },
            Stmt::Putc(expr) => {
                let folded = self.fold_expr(expr);
    
                match folded {
                    Expr::Variable(name) => {
                        // If it's a virtual array, print sequentially
                        let array_data = if let Some((_, _, elem_type, literals)) = self.arrays.get(&name) {
                            if *elem_type == Type::Char {
                                literals.clone().map(|lits| (name.clone(), lits))
                            } else { None }
                        } else { None };

                        if let Some((name, lits)) = array_data {
                            for (_i, lit) in lits.iter().enumerate() {
                                self.materialize_to_cell(&name, lit.clone(), false);
                                self.indent();
                                self.emit_line(&format!("print {}", name));
                            }
                        } else {
                            // Single physical variable or unknown
                            self.indent();
                            self.emit_line(&format!("print {}", name));
                        }
                    },
                    Expr::ArrayAccess { array, index } => {
                        // If it's a physical cell array access, we can print the cell name directly
                        if let Expr::Number(i) = index.as_ref() {
                            if let Expr::Variable(array_name) = array.as_ref() {
                                if let Some((_, _, elem_type, _)) = self.arrays.get(array_name) {
                                    if !elem_type.is_virtual() {
                                        let cell_name = self.get_array_var_name(array_name, *i);
                                        self.indent();
                                        self.emit_line(&format!("print {}", cell_name));
                                        return;
                                    }
                                }
                            }
                        }
            
                        // Otherwise fall back to materialization
                        self.materialize_to_cell("__hbf_tmp", Expr::ArrayAccess { array, index }, false);
                        self.indent();
                        self.emit_line("print __hbf_tmp");
                    },
                    Expr::Number(n) => {
                        self.indent();
                        self.emit_line(&format!("print {}", n));
                    },
                    Expr::CharLiteral(c) => {
                        self.indent();
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
                        // Complex expression materialization
                        // Use local 'tmp' cell for BFO printing
                        self.materialize_to_cell("__hbf_tmp", folded, false);
                        self.indent();
                        self.emit_line("print __hbf_tmp");
                    }
                }
            },
            Stmt::For { init, condition, update, body } => {
                // for(;;) is strictly for unfolding (simulation)
                self.push_scope();
                if let Some(i) = init {
                    self.gen_stmt(*i.clone(), false);
                }
    
                let mut iterations = 0;
                while iterations < 10000 {
                    let cond_val = if let Some(cond) = &condition {
                        let folded = self.fold_expr(cond.clone());
                        match folded {
                            Expr::BoolLiteral(b) => b,
                            Expr::Number(n) => n != 0,
                            _ => {
                                panic!("For loop condition must be a compile-time constant for unfolding, got {:?}", folded);
                            }
                        }
                    } else {
                        true
                    };
        
                    if !cond_val { break; }
        
                    for s in &body {
                        self.gen_stmt(s.clone(), false);
                    }
        
                    if let Some(ref u) = update {
                        self.gen_stmt(*u.clone(), false);
                    }
                    iterations += 1;
                }
    
                if iterations >= 10000 {
                    panic!("Loop unrolling exceeded limit (possible infinite loop or too large)");
                }
                self.pop_scope();
            },
            Stmt::Forn { count, body } => {
                let folded_count = self.fold_expr(count.clone());
    
                // Determine the loop counter name
                let mut is_var = false;
                let name = match &count {
                    Expr::Variable(n) => {
                        if let Some(val) = self.get_variable(n) {
                            // Generate an anonymous counter
                            let for_name = format!("__hbf_forn_{}", self.forn_counter);
                            self.forn_counter += 1;
                            is_var = true;
                            for_name
                        } else {
                            n.clone()
                        }
                    },
                    _ => {
                        // Generate an anonymous counter
                        let n = format!("__hbf_forn_{}", self.forn_counter);
                        self.forn_counter += 1;
                        is_var = true;
                        n
                    }
                };

                // Generate native countdown loop: set n value; while n { body; sub n 1 }
                if is_var { 
                    self.materialize_to_cell(&name, folded_count, true); 
                } else {
                    self.materialize_to_cell(&name, folded_count, false); 
                }
    
                self.indent();
                self.emit_line(&format!("while {} {{", name));
    
                self.indent_level += 1;
                self.native_loop_depth += 1;

                for s in body {
                    self.gen_stmt(s, false);
                }
                self.native_loop_depth -= 1;
    
                // Decrement counter
                self.emit_sub(&name, "1");
                self.indent_level -= 1;
    
                self.indent();
                self.emit_line("}");
                if is_var {
                    self.indent();
                    self.emit_line(&format!("free {}", name));
                }
                if is_var {
                    self.forn_counter -= 1;
                }
            },
            Stmt::While { condition, body } => {
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
                    _ => panic!("Unsupported while condition"),
                }
                self.emit_line(" {");
    
                self.indent_level += 1;
                self.native_loop_depth += 1;
    
                for s in body {
                    self.gen_stmt(s, false);
                }
                self.native_loop_depth -= 1;
                self.indent_level -= 1;
    
                self.indent();
                self.emit_line("}");
            },
            Stmt::ExprStmt(expr) => {
                if let Expr::FunctionCall { name, args } = expr {
                    if let Some(func_def) = self.functions.get(&name).cloned() {
                        if let Stmt::FuncDecl { params, body, .. } = func_def {
                             self.inline_function(params, args, body);
                        }
                    } else {
                        // Function not found, or maybe it's a native/extern?
                        // For now we assume strict simulation or existing funcs.
                        panic!("Undefined function call: {}", name);
                    }
                }
            },
            Stmt::If { condition, then_branch, else_branch } => {
                let folded_cond = self.fold_expr(condition);
                match folded_cond {
                    Expr::BoolLiteral(b) => {
                        if b {
                            for s in then_branch { self.gen_stmt(s, false); }
                        } else if let Some(else_stmts) = else_branch {
                            for s in else_stmts { self.gen_stmt(s, false); }
                        }
                    },
                    Expr::Number(n) => {
                        if n != 0 {
                            for s in then_branch { self.gen_stmt(s, false); }
                        } else if let Some(else_stmts) = else_branch {
                            for s in else_stmts { self.gen_stmt(s, false); }
                        }
                    },
                    _ => {
                        // Runtime IF - convert to single-execution while loop
                        let cond_name = format!("__hbf_if_cond_{}", self.forn_counter);
                        self.forn_counter += 1;
                        self.materialize_to_cell(&cond_name, folded_cond, true);
            
                        if let Some(else_stmts) = else_branch {
                            let else_name = format!("__hbf_else_flag_{}", self.forn_counter);
                            self.forn_counter += 1;
                            self.emit_new(&else_name, "1");
                
                            self.indent();
                            self.emit_line(&format!("while {} {{", cond_name));
                            self.indent_level += 1;
                            for s in then_branch {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&cond_name, "0");
                            self.emit_set(&else_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&cond_name);
                            self.indent();
                            self.emit_line(&format!("while {} {{", else_name));
                            self.indent_level += 1;
                            for s in else_stmts {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&else_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&else_name);
                        } else {
                            self.indent();
                            self.emit_line(&format!("while {} {{", cond_name));
                            self.indent_level += 1;
                            for s in then_branch {
                                self.gen_stmt(s, false);
                            }
                            self.emit_set(&cond_name, "0");
                            self.indent_level -= 1;
                            self.indent();
                            self.emit_line("}");
                            self.free_cell(&cond_name);
                        }
                    }
                }
            },
            Stmt::Group(stmts) => {
                for s in stmts {
                    self.gen_stmt(s, false);
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
            Expr::BoolLiteral(b) => self.emit(&format!("{}", if b { 1 } else { 0 })),
            Expr::Variable(name) => {
                if let Some(val) = self.get_variable(&name) {
                    // Recursively resolve in case of nested virtuals
                    self.gen_expr(val.clone());
                } else {
                    self.emit(&name);
                }
            },
            Expr::FunctionCall { name, .. } => {
                panic!("Function calls in expressions are not supported in simulation-only mode: {}. They must be foldable to constants.", name);
            },
            Expr::ArrayAccess { array, index } => {
                if let Expr::Number(i) = &*index {
                    match &*array {
                        Expr::Variable(name) => {
                            // If it's a streaming array, we can return its literal value if known
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if elem_type.is_virtual() {
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
            Expr::BoolLiteral(b) => {
                self.emit(&format!("{}", if b { 1 } else { 0 }));
            },
            Expr::Variable(name) => {
                if let Some(val) = self.get_variable(&name) {
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
                                if elem_type.is_virtual() {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        self.gen_expr_simple(lit.clone());
                                        return;
                                    }
                                }
                            }
                            // If it's a physical cell array, we emit the cell name (e.g. arr_1)
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



    fn fold_expr(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Variable(name) => {
                // Substitute variable with its value if known (search scope stack)
                if let Some(value) = self.get_variable(&name) {
                    value
                } else {
                    Expr::Variable(name)
                }
            },
            Expr::BinaryOp { left, op, right } => {
                let left_folded = self.fold_expr(*left);
                let right_folded = self.fold_expr(*right);
                
                // Helper to get numeric value of Number or CharLiteral or BoolLiteral
                let to_num = |e: &Expr| match e {
                    Expr::Number(n) => Some(*n),
                    Expr::CharLiteral(c) => Some(*c as i32),
                    Expr::BoolLiteral(b) => Some(if *b { 1 } else { 0 }),
                    _ => None,
                };

                // Try to evaluate constant expressions
                if let (Some(l), Some(r)) = (to_num(&left_folded), to_num(&right_folded)) {
                    match op {
                        Token::Plus => return Expr::Number(l + r),
                        Token::Minus => return Expr::Number(l - r),
                        Token::Star => return Expr::Number(l * r),
                        Token::Slash => {
                            if r == 0 { panic!("Division by zero in constant folding"); }
                            return Expr::Number(l / r);
                        },
                        Token::Percent => {
                            if r == 0 { panic!("Modulo by zero in constant folding"); }
                            return Expr::Number(l % r);
                        },
                        Token::DoubleEquals => return Expr::BoolLiteral(l == r),
                        Token::NotEquals => return Expr::BoolLiteral(l != r),
                        Token::Less => return Expr::BoolLiteral(l < r),
                        Token::LessEqual => return Expr::BoolLiteral(l <= r),
                        Token::Greater => return Expr::BoolLiteral(l > r),
                        Token::GreaterEqual => return Expr::BoolLiteral(l >= r),
                        Token::AndAnd => return Expr::BoolLiteral((l != 0) && (r != 0)),
                        Token::OrOr => return Expr::BoolLiteral((l != 0) || (r != 0)),
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
            Expr::ArrayAccess { array, index } => {
                let array_folded = self.fold_expr(*array);
                let index_folded = self.fold_expr(*index);

                if let Expr::Number(i) = &index_folded {
                    match &array_folded {
                        Expr::StringLiteral(s) => {
                            if let Some(ch) = s.chars().nth(*i as usize) {
                                return Expr::CharLiteral(ch);
                            }
                        },
                        Expr::ArrayLiteral(elements) => {
                            if let Some(el) = elements.get(*i as usize) {
                                return el.clone();
                            }
                        },
                        Expr::Variable(name) => {
                            if let Some((_, _, elem_type, Some(literals))) = self.arrays.get(name) {
                                if *elem_type != Type::Cell {
                                    if let Some(lit) = literals.get(*i as usize) {
                                        return lit.clone();
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }

                Expr::ArrayAccess {
                    array: Box::new(array_folded),
                    index: Box::new(index_folded),
                }
            },
            Expr::MemberAccess { object, member } => {
                let object_folded = self.fold_expr(*object);
                if member == "length" {
                    match &object_folded {
                        Expr::StringLiteral(s) => return Expr::Number(s.len() as i32),
                        Expr::ArrayLiteral(elements) => return Expr::Number(elements.len() as i32),
                        Expr::Variable(name) => {
                            if let Some((_, len, _, _)) = self.arrays.get(name) {
                                return Expr::Number(*len as i32);
                            }
                        },
                        _ => {}
                    }
                }
                Expr::MemberAccess {
                    object: Box::new(object_folded),
                    member,
                }
            },
            _ => expr, // Other expressions (CharLiteral, StringLiteral, Number) unchanged
        }
    }



    fn inline_function(&mut self, params: Vec<(Type, String)>, args: Vec<Expr>, body: Vec<Stmt>) {
        // 1. Evaluate arguments in CURRENT scope
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.fold_expr(arg));
        }

        // 2. Push NEW scope for the function body (virtual scope)
        self.push_scope();
        
        // 3. Start BFO block (runtime scope)
        self.indent();
        self.emit_line("{");
        self.indent_level += 1;

        // 4. Initialize parameters
        for (i, (param_type, param_name)) in params.iter().enumerate() {
            if let Some(arg) = evaluated_args.get(i) {
                match param_type {
                    Type::Cell => {
                        // Pass-by-value: Create new local cell initialized with argument value
                        // This 'new' will happen inside the BFO block, shadowing any outer 'param_name'
                        self.materialize_to_cell(param_name, arg.clone(), true);
                    },
                    _ => {
                        // Virtual variables (int, char): simulate via aliasing/folding
                        self.set_variable(param_name, arg.clone());
                    }
                }
            }
        }

        // 5. Generate code for the body
        for stmt in body {
            self.gen_stmt(stmt, false);
        }

        // 6. End BFO block
        self.indent_level -= 1;
        self.indent();
        self.emit_line("}");

        // 7. Pop scope
        self.pop_scope();
    }

    fn get_var_type(&self, _name: &str) -> Option<Type> {
        // ... this might need scoping too if types can change, but usually they don't in HBF
        None
    }
}
