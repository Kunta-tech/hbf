use crate::bfo_ast::*;
use crate::ir::BFO;
use std::collections::{HashMap, HashSet};

pub struct BFOCompiler {
    instructions: Vec<BFO>,
    variables: Vec<(String, usize, usize, bool)>, // (name, address, size, owned)
    scope_marks: Vec<usize>,                       // indices into variables stack
    functions: HashMap<String, (Vec<String>, Vec<BFOStmt>)>,
    current_pointer: usize,
    next_free_cell: usize,
    free_pool: Vec<(usize, usize)>, // (address, size)
    touched_cells: HashSet<usize>,
}

impl BFOCompiler {
    pub fn new() -> Self {
        BFOCompiler {
            instructions: Vec::new(),
            variables: Vec::new(),
            scope_marks: vec![0],
            functions: HashMap::new(),
            current_pointer: 0,
            next_free_cell: 0,
            free_pool: Vec::new(),
            touched_cells: HashSet::new(),
        }
    }

    pub fn compile(&mut self, program: BFOProgram, base_dir: &std::path::Path) -> Vec<BFO> {
        let all_items = self.flatten_program(program, base_dir);
        
        // First pass: collect functions
        for item in &all_items {
            if let BFOItem::Function { name, params, body } = item {
                self.functions.insert(name.clone(), (params.clone(), body.clone()));
            }
        }

        // Second pass: generate code for top-level items
        for item in all_items {
            match item {
                BFOItem::Statement(stmt) => {
                    self.compile_stmt(stmt);
                }
                _ => {}
            }
        }

        self.instructions.clone()
    }

    fn flatten_program(&mut self, program: BFOProgram, current_dir: &std::path::Path) -> Vec<BFOItem> {
        let mut flattened = Vec::new();
        for item in program.items {
            if let BFOItem::Include(path) = item {
                let full_path = current_dir.join(&path);
                let source = std::fs::read_to_string(&full_path).expect(&format!("Failed to read include file: {}", full_path.display()));
                let lexer = crate::bfo_lexer::BFOLexer::new(&source);
                let mut parser = crate::bfo_parser::BFOParser::new(lexer);
                let sub_program = parser.parse();
                let sub_dir = full_path.parent().unwrap_or(std::path::Path::new("."));
                flattened.extend(self.flatten_program(sub_program, sub_dir));
            } else {
                flattened.push(item);
            }
        }
        flattened
    }

    fn emit(&mut self, op: BFO) {
        self.touched_cells.insert(self.current_pointer);
        self.instructions.push(op);
    }

    fn move_to(&mut self, target: usize) {
        if target != self.current_pointer {
            self.emit(BFO::Shift(target as isize - self.current_pointer as isize));
            self.current_pointer = target;
        }
    }

    fn get_var_info(&self, name: &str) -> (usize, usize) {
        self.variables.iter().rev()
            .find(|(n, _, _, _)| n == name)
            .map(|(_, addr, size, _)| (*addr, *size))
            .expect(&format!("Undefined variable in BFO: {}", name))
    }

    fn resolve_value(&self, value: &BFOValue) -> (usize, usize) {
        match value {
            BFOValue::Variable(v, offset) => {
                let (addr, size) = self.get_var_info(v);
                if *offset >= size {
                    panic!("Offset {} out of bounds for variable {} (size {})", offset, v, size);
                }
                (addr + *offset, size - *offset)
            }
            BFOValue::Number(n) => (*n as usize, 1),
            BFOValue::Char(c) => (*c as usize, 1),
        }
    }

    fn auto_free_scope(&mut self) {
        let mark = self.scope_marks.pop().expect("Underflow in scope marks");
        while self.variables.len() > mark {
            let (_, addr, size, owned) = self.variables.pop().unwrap();
            if owned {
                self.free_pool.push((addr, size));
            }
        }
        self.merge_free_pool();
    }

    fn prepare_call_arg(&mut self, arg: &BFOValue, temps: &mut Vec<(usize, usize)>) -> (usize, usize) {
        match arg {
            BFOValue::Variable(v, offset) => {
                let (addr, size) = self.get_var_info(v);
                if *offset >= size {
                    panic!("Offset {} out of bounds for variable {} (size {})", offset, v, size);
                }
                (addr + *offset, size - *offset)
            }
            BFOValue::Number(n) => self.allocate_temp_const(*n as i16, temps),
            BFOValue::Char(c) => self.allocate_temp_const(*c as i16, temps),
        }
    }

    fn allocate_temp_const(&mut self, val: i16, temps: &mut Vec<(usize, usize)>) -> (usize, usize) {
        let addr = self.allocate_segment(1);
        let saved_ptr = self.current_pointer;
        self.move_to(addr);
        if self.touched_cells.contains(&addr) {
            self.emit(BFO::Clear);
        }
        self.emit(BFO::Modify(val));
        self.move_to(saved_ptr);
        temps.push((addr, 1));
        (addr, 1)
    }

    fn allocate_segment(&mut self, size: usize) -> usize {
        // LIFO free pool allocation: check the last element first
        if let Some(idx) = self.free_pool.iter().rposition(|&(_, f_size)| f_size >= size) {
            let (f_addr, f_size) = self.free_pool.remove(idx);
            if f_size > size {
                // Return remainder to pool (LIFO)
                self.free_pool.push((f_addr + size, f_size - size));
                self.merge_free_pool();
            }
            f_addr
        } else {
            let addr = self.next_free_cell;
            self.next_free_cell += size;
            addr
        }
    }

    fn merge_free_pool(&mut self) {
        if self.free_pool.len() < 2 {
            return;
        }
        self.free_pool.sort_by_key(|&(addr, _)| addr);
        
        let mut merged = Vec::new();
        let (mut cur_addr, mut cur_size) = self.free_pool[0];
        
        for i in 1..self.free_pool.len() {
            let (next_addr, next_size) = self.free_pool[i];
            if cur_addr + cur_size == next_addr {
                cur_size += next_size;
            } else {
                merged.push((cur_addr, cur_size));
                cur_addr = next_addr;
                cur_size = next_size;
            }
        }
        merged.push((cur_addr, cur_size));
        self.free_pool = merged;
    }

    fn compile_stmt(&mut self, stmt: BFOStmt) {
        match stmt {
            BFOStmt::New { name, size } => {
                let cell = self.allocate_segment(size);
                self.variables.push((name, cell, size, true));
                
                let saved_ptr = self.current_pointer;
                for i in 0..size {
                    let addr = cell + i;
                    if self.touched_cells.contains(&addr) {
                        self.move_to(addr);
                        self.emit(BFO::Clear);
                    }
                }
                self.move_to(saved_ptr);
            }
            BFOStmt::Free { name } => {
                let idx = self.variables.iter().rposition(|(n, _, _, _)| n == &name);
                if let Some(idx) = idx {
                    let mut reclaimed = None;
                    if let Some((_, addr, size, ref mut owned)) = self.variables.get_mut(idx) {
                        if *owned {
                            reclaimed = Some((*addr, *size));
                            *owned = false;
                        }
                    }
                    if let Some((addr, size)) = reclaimed {
                        self.free_pool.push((addr, size));
                        self.merge_free_pool();
                    }
                }
            }
            BFOStmt::Goto { value } => {
                let (addr, _) = self.resolve_value(&value);
                self.move_to(addr);
            }
            BFOStmt::At(value) => {
                let (addr, _) = self.resolve_value(&value);
                self.current_pointer = addr;
                self.emit(BFO::ForceGoto(addr));
            }
            BFOStmt::Shift(n) => {
                self.emit(BFO::Shift(n));
                if n > 0 {
                    self.current_pointer += n as usize;
                } else {
                    self.current_pointer = self.current_pointer.saturating_sub(n.unsigned_abs());
                }
            }
            BFOStmt::Modify(n) => {
                self.emit(BFO::Modify(n));
            }
            BFOStmt::Set(n) => {
                if n == 0 {
                    self.emit(BFO::Clear);
                } else {
                    self.emit(BFO::Clear);
                    if n <= 128 {
                        self.emit(BFO::Modify(n as i16));
                    } else {
                        self.emit(BFO::Modify(-(256i16 - n as i16)));
                    }
                }
            }
            BFOStmt::Print => {
                self.emit(BFO::Print);
            }
            BFOStmt::Scan => {
                self.emit(BFO::Scan);
            }
            BFOStmt::Loop(body) => {
                let saved_instructions = std::mem::replace(&mut self.instructions, Vec::new());
                
                for s in body {
                    self.compile_stmt(s);
                }
                
                let loop_instructions = std::mem::replace(&mut self.instructions, saved_instructions);
                self.emit(BFO::Loop(loop_instructions));
            }
            BFOStmt::Call { name, args } => {
                if let Some((params, body)) = self.functions.get(&name).cloned() {
                    let mut temps_to_free = Vec::new();
                    self.scope_marks.push(self.variables.len());

                    for (i, param_name) in params.iter().enumerate() {
                        if i < args.len() {
                            let info = self.prepare_call_arg(&args[i], &mut temps_to_free);
                            self.variables.push((param_name.clone(), info.0, info.1, false));
                        }
                    }

                    for s in body {
                        self.compile_stmt(s);
                    }
                    
                    self.auto_free_scope();

                    for (addr, size) in temps_to_free {
                        self.free_pool.push((addr, size));
                    }
                    self.merge_free_pool();
                }
            }
            BFOStmt::Block(stmts) => {
                self.scope_marks.push(self.variables.len());
                for s in stmts {
                    self.compile_stmt(s);
                }
                self.auto_free_scope();
            }
            BFOStmt::Alias { name, value } => {
                let (addr, size) = self.resolve_value(&value);
                self.variables.push((name, addr, size, false));
            }
        }
    }
}
