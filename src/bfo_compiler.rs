
use crate::bfo_ast::{BFOProgram, BFOItem, BFOValue, BFOStmt};
use crate::ir::BFO;
use std::collections::HashMap;

pub struct BFOCompiler {
    instructions: Vec<BFO>,
    symbol_table: HashMap<String, usize>,
    functions: HashMap<String, (Vec<String>, Vec<BFOStmt>)>,
    current_pointer: usize,
    next_free_cell: usize,
    free_pool: Vec<usize>,
}

impl BFOCompiler {
    pub fn new() -> Self {
        BFOCompiler {
            instructions: Vec::new(),
            symbol_table: HashMap::new(),
            functions: HashMap::new(),
            current_pointer: 0,
            next_free_cell: 0,
            free_pool: Vec::new(),
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
            cell
        } else {
            let cell = self.next_free_cell;
            self.next_free_cell += 1;
            cell
        }
    }

    fn free_cell(&mut self, cell: usize) {
        self.free_pool.push(cell);
    }

    fn get_or_allocate_cell(&mut self, name: &str) -> usize {
        if let Some(&cell) = self.symbol_table.get(name) {
            cell
        } else {
            let cell = self.allocate_cell();
            self.symbol_table.insert(name.to_string(), cell);
            cell
        }
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
                let src = self.get_or_allocate_cell(&name);
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
            let old_table = self.symbol_table.clone();
            let mut local_allocations = Vec::new();
            
            for (i, param) in params.iter().enumerate() {
                if let Some(arg) = args.get(i) {
                    match arg {
                        BFOValue::Variable(v) => {
                            let cell = *old_table.get(v).expect("Undefined arg");
                            self.symbol_table.insert(param.clone(), cell);
                        }
                        BFOValue::Number(_) | BFOValue::Char(_) => {
                            // Materialize literal to a temp cell for the param
                            let cell = self.allocate_cell();
                            local_allocations.push(cell);
                            self.move_to(cell);
                            self.emit(BFO::Clear);
                            self.emit_value_into(arg.clone(), cell);
                            self.symbol_table.insert(param.clone(), cell);
                        }
                    }
                }
            }

            for stmt in body {
                self.compile_stmt(stmt);
            }

            self.symbol_table = old_table;
            
            // Explicitly free literal cells used as params
            for cell in local_allocations {
                self.free_cell(cell);
            }
        } else {
            panic!("Undefined function call in BFO: {}", name);
        }
    }

    fn compile_stmt(&mut self, stmt: BFOStmt) {
        match stmt {
            BFOStmt::Set { name, value } => {
                let cell = self.get_or_allocate_cell(&name);
                self.move_to(cell);
                self.emit(BFO::Clear);
                self.emit_value_into(value, cell);
            }
            BFOStmt::Add { name, value } => {
                let cell = self.get_or_allocate_cell(&name);
                self.emit_value_into(value, cell);
            }
            BFOStmt::Sub { name, value } => {
                let cell = self.get_or_allocate_cell(&name);
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
                        let src = self.get_or_allocate_cell(&v);
                        self.sub_variable(src, cell);
                    }
                }
            }
            BFOStmt::Print { value } => {
                match value {
                    BFOValue::Variable(name) => {
                        let cell = self.get_or_allocate_cell(&name);
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
                let cond_cell = self.get_or_allocate_cell(&condition);
                self.move_to(cond_cell);
                
                let parent_instructions = std::mem::replace(&mut self.instructions, Vec::new());
                for s in body {
                    self.compile_stmt(s);
                }
                // BF Loop requires head to be at cond_cell at the start and end of block.
                self.move_to(cond_cell);
                
                let loop_ops = std::mem::replace(&mut self.instructions, parent_instructions);
                self.emit(BFO::Loop(loop_ops));
            }
            BFOStmt::Call { name, args } => {
                self.handle_call(&name, args);
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
