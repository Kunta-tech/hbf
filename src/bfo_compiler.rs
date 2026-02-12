
use crate::bfo_ast::{BFOProgram, BFOItem, BFOValue, BFOStmt};
use crate::ir::BFO;
use std::collections::HashMap;

pub struct BFOCompiler {
    instructions: Vec<BFO>,
    scopes: Vec<HashMap<String, usize>>,
    functions: HashMap<String, (Vec<String>, Vec<BFOStmt>)>,
    current_pointer: usize,
    next_free_cell: usize,
    free_pool: Vec<usize>,
}

impl BFOCompiler {
    pub fn new() -> Self {
        BFOCompiler {
            instructions: Vec::new(),
            scopes: vec![HashMap::new()], // global scope
            functions: HashMap::new(),
            current_pointer: 0,
            next_free_cell: 0,
            free_pool: Vec::new(),
        }
    }
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for (_, cell) in scope {
                self.free_cell(cell);
            }
        }
    }

    pub fn compile(&mut self, program: BFOProgram) -> Vec<BFO> {
        // First pass: collect functions
        for item in &program.items {
            if let BFOItem::Function { name, params, body } = item {
                self.functions.insert(name.clone(), (params.clone(), body.clone()));
            }
        }

        // Second pass: generate code for top-level items
        for item in program.items {
            match item {
                BFOItem::Statement(stmt) => {
                    self.compile_stmt(stmt);
                }
                BFOItem::Function { .. } => {} // Handled in first pass
            }
        }

        self.instructions.clone()
    }

    fn emit(&mut self, op: BFO) {
        self.instructions.push(op);
    }

    fn allocate_cell(&mut self) -> usize {
        if let Some(cell) = self.free_pool.pop() {
            // println!("Allocating cell {} from pool", cell);
            cell
        } else {
            let cell = self.next_free_cell;
            self.next_free_cell += 1;
            // println!("Allocating cell {} from next_free_cell", cell);
            cell
        }
    }

    fn free_cell(&mut self, cell: usize) {
        // println!("Freeing cell {}", cell);
        self.free_pool.push(cell);
    }

    fn get_or_allocate_cell(&mut self, name: &str) -> usize {
        // Search from inner scope → outer
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(&cell) = scope.get(name) {
                return cell;
            }
        }

        // Not found → allocate in current scope
        let cell = self.allocate_cell();
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), cell);

        cell
    }

    fn get_cell(&mut self, name: &str) -> Option<usize> {
        // Search from inner scope → outer
        for scope in self.scopes.iter().rev() {
            if let Some(&cell) = scope.get(name) {
                return Some(cell);
            }
        }
        None
    }


    fn move_to(&mut self, target: usize) {
        if target > self.current_pointer {
            self.emit(BFO::MoveRight(target - self.current_pointer));
        } else if target < self.current_pointer {
            self.emit(BFO::MoveLeft(self.current_pointer - target));
        }
        self.current_pointer = target;
    }

    fn emit_value_into(&mut self, value: BFOValue, target: usize) {
        match value {
            BFOValue::Number(n) => {
                self.move_to(target);
                if n > 0 {
                    self.emit(BFO::Add(n as u8));
                } else if n < 0 {
                    self.emit(BFO::Sub((-n) as u8));
                }
            }
            BFOValue::Char(c) => {
                self.move_to(target);
                self.emit(BFO::Add(c as u8));
            }
            BFOValue::Variable(name) => {
                let src = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                self.copy_cell(src, target);
            }
        }
    }

    fn copy_cell(&mut self, src: usize, dest: usize) {
        if src == dest { return; }
        
        // BF non-destructive copy using 1 temp cell
        let temp = self.allocate_cell();

        // Ensure temp is clear
        self.move_to(temp);
        self.emit(BFO::Clear);

        // Move src to dest and temp
        self.move_to(src);
        let mut move_body = Vec::new();
        move_body.push(BFO::Sub(1));
        
        // Add to dest
        let diff_dest = if dest > src { BFO::MoveRight(dest - src) } else { BFO::MoveLeft(src - dest) };
        move_body.push(diff_dest.clone());
        move_body.push(BFO::Add(1));
        
        // Add to temp
        let diff_temp = if temp > dest { BFO::MoveRight(temp - dest) } else { BFO::MoveLeft(dest - temp) };
        move_body.push(diff_temp.clone());
        move_body.push(BFO::Add(1));
        
        // Back to src
        let rev_temp = if temp > src { BFO::MoveLeft(temp - src) } else { BFO::MoveRight(src - temp) };
        move_body.push(rev_temp);

        self.emit(BFO::Loop(move_body));

        // Restore src from temp
        self.move_to(temp);
        let mut restore_body = Vec::new();
        restore_body.push(BFO::Sub(1));
        
        let diff_src = if temp > src { BFO::MoveLeft(temp - src) } else { BFO::MoveRight(src - temp) };
        restore_body.push(diff_src.clone());
        restore_body.push(BFO::Add(1));
        
        let rev_src = if temp > src { BFO::MoveRight(temp - src) } else { BFO::MoveLeft(src - temp) };
        restore_body.push(rev_src);
        
        self.emit(BFO::Loop(restore_body));

        self.free_cell(temp); // Reuse temp for future operations
    }

    fn handle_call(&mut self, name: &str, args: Vec<BFOValue>) {
        if let Some((params, body)) = self.functions.get(name).cloned() {

            // Create function scope
            self.enter_scope();

            for (i, param) in params.iter().enumerate() {
                if let Some(arg) = args.get(i) {

                    let cell = match arg {
                        BFOValue::Variable(v) => {
                            // Use your existing scoped resolver
                            self.get_cell(v).expect(&format!("Undefined argument variable: {}", v))
                        }

                        BFOValue::Number(_) | BFOValue::Char(_) => {
                            let cell = self.allocate_cell();

                            self.move_to(cell);
                            self.emit(BFO::Clear);
                            self.emit_value_into(arg.clone(), cell);

                            cell
                        }
                    };

                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(param.clone(), cell);
                }
            }

            for stmt in body {
                self.compile_stmt(stmt);
            }

            // Scope exit frees everything
            self.exit_scope();

        } else {
            panic!("Undefined function call in BFO: {}", name);
        }
    }

    fn compile_stmt(&mut self, stmt: BFOStmt) {
        match stmt {
            BFOStmt::Set { name, value } => {
                let cell = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                self.move_to(cell);
                self.emit(BFO::Clear);
                self.emit_value_into(value, cell);
            }
            BFOStmt::New { name, value } => {
                let cell = self.get_or_allocate_cell(&name);
                self.move_to(cell);
                self.emit(BFO::Clear);
                self.emit_value_into(value, cell);
            }
            BFOStmt::Add { name, value } => {
                let cell = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                self.emit_value_into(value, cell);
            }
            BFOStmt::Sub { name, value } => {
                let cell = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                match value {
                    BFOValue::Number(n) => {
                        self.move_to(cell);
                        self.emit(BFO::Sub(n as u8));
                    }
                    BFOValue::Char(c) => {
                        self.move_to(cell);
                        self.emit(BFO::Sub(c as u8));
                    }
                    BFOValue::Variable(v) => {
                        let src = self.get_cell(&v).expect(&format!("Undefined variable: {}", v));
                        self.sub_variable(src, cell);
                    }
                }
            }
            BFOStmt::Print { value } => {
                match value {
                    BFOValue::Variable(name) => {
                        let cell = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                        self.move_to(cell);
                        self.emit(BFO::Print);
                    }
                    BFOValue::Number(n) => {
                        // Temp cell for printing literal
                        let cell = self.allocate_cell();
                        self.move_to(cell);
                        self.emit(BFO::Clear);
                        self.emit(BFO::Add(n as u8));
                        self.emit(BFO::Print);
                        self.free_cell(cell);
                    }
                    BFOValue::Char(c) => {
                        let cell = self.allocate_cell();
                        self.move_to(cell);
                        self.emit(BFO::Clear);
                        self.emit(BFO::Add(c as u8));
                        self.emit(BFO::Print);
                        self.free_cell(cell);
                    }
                }
            }
            BFOStmt::While { condition, body } => {
                let cond_cell = self.get_cell(&condition).expect(&format!("Undefined variable: {}", condition));
                self.move_to(cond_cell);
                
                let parent_instructions = std::mem::replace(&mut self.instructions, Vec::new());
                self.enter_scope();
                for s in body {
                    self.compile_stmt(s);
                }
                self.exit_scope();
                // BF Loop requires head to be at cond_cell at the start and end of block.
                self.move_to(cond_cell);
                
                let loop_ops = std::mem::replace(&mut self.instructions, parent_instructions);
                self.emit(BFO::Loop(loop_ops));
            }
            BFOStmt::Call { name, args } => {
                self.handle_call(&name, args);
            }
            BFOStmt::Free { name } => {
                let cell = self.get_cell(&name).expect(&format!("Undefined variable: {}", name));
                self.free_cell(cell);
            }
            BFOStmt::Ref { alias, original } => {
                let cell = self.get_cell(&original).expect(&format!("Undefined variable to ref: {}", original));
                self.scopes.last_mut().unwrap().insert(alias, cell);
            }
            BFOStmt::Scan { name } => {
                let cell = self.get_cell(&name).expect(&format!("Undefined variable for scan: {}", name));
                self.move_to(cell);
                self.emit(BFO::Scan);
            }
            BFOStmt::Move { dest, src } => {
                let dest_cell = self.get_cell(&dest).expect(&format!("Undefined variable: {}", dest));
                let src_cell = self.get_cell(&src).expect(&format!("Undefined variable: {}", src));
                
                if dest_cell == src_cell { return; }

                // 1. Clear dest
                self.move_to(dest_cell);
                self.emit(BFO::Clear);

                // 2. Move src to dest: while src { sub src 1; move_to dest; add dest 1; move_to src }
                self.move_to(src_cell);
                let mut body = Vec::new();
                body.push(BFO::Sub(1));
                
                let diff = if dest_cell > src_cell {
                    BFO::MoveRight(dest_cell - src_cell)
                } else {
                    BFO::MoveLeft(src_cell - dest_cell)
                };
                body.push(diff.clone());
                body.push(BFO::Add(1));
                
                let rev = if dest_cell > src_cell {
                    BFO::MoveLeft(dest_cell - src_cell)
                } else {
                    BFO::MoveRight(src_cell - dest_cell)
                };
                body.push(rev);
                
                self.emit(BFO::Loop(body));
            }
            BFOStmt::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                self.exit_scope();
            }
        }
    }

    fn sub_variable(&mut self, src: usize, dest: usize) {
        // Destructive subtract src from dest. 
        // Note: src will be 0 after this.
        // If we want non-destructive, we'd need a temp cell.
        // BFO `sub` is typically used for things like `sub n 1` where n is counter.
        // If it's a variable-to-variable sub, we should probably be non-destructive.
        
        let temp = self.allocate_cell();
        self.move_to(temp);
        self.emit(BFO::Clear);

        self.move_to(src);
        let mut sub_body = Vec::new();
        sub_body.push(BFO::Sub(1));

        let diff_dest = if dest > src { BFO::MoveRight(dest - src) } else { BFO::MoveLeft(src - dest) };
        sub_body.push(diff_dest.clone());
        sub_body.push(BFO::Sub(1));
        
        let diff_temp = if temp > dest { BFO::MoveRight(temp - dest) } else { BFO::MoveLeft(dest - temp) };
        sub_body.push(diff_temp.clone());
        sub_body.push(BFO::Add(1));
        
        let rev_temp = if temp > src { BFO::MoveLeft(temp - src) } else { BFO::MoveRight(src - temp) };
        sub_body.push(rev_temp);

        self.emit(BFO::Loop(sub_body));

        // Restore src from temp
        self.move_to(temp);
        let mut restore_body = Vec::new();
        restore_body.push(BFO::Sub(1));
        
        let diff_src = if temp > src { BFO::MoveLeft(temp - src) } else { BFO::MoveRight(src - temp) };
        restore_body.push(diff_src.clone());
        restore_body.push(BFO::Add(1));
        
        let rev_src = if temp > src { BFO::MoveRight(temp - src) } else { BFO::MoveLeft(src - temp) };
        restore_body.push(rev_src);
        
        self.emit(BFO::Loop(restore_body));

        self.free_cell(temp);
    }
}
