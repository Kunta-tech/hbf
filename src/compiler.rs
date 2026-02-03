
use crate::ast::{Expr, Stmt, Program};
use crate::ir::BFO;
use crate::token::Token;
use std::collections::HashMap;

pub struct Compiler {
    pub instructions: Vec<BFO>,
    symbol_table: HashMap<String, usize>, // Variable name -> Cell Index
    current_pointer: usize, // Current implementation pointer location on tape
    next_free_cell: usize,  // Allocator
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Vec::new(),
            symbol_table: HashMap::new(),
            current_pointer: 0,
            next_free_cell: 0,
        }
    }

    pub fn compile(&mut self, program: Program) -> Vec<BFO> {
        for stmt in program.statements {
            self.compile_stmt(stmt);
        }
        self.instructions.clone()
    }

    fn emit(&mut self, op: BFO) {
        self.instructions.push(op);
    }

    fn move_to(&mut self, target_cell: usize) {
        if target_cell > self.current_pointer {
            let diff = target_cell - self.current_pointer;
            self.emit(BFO::MoveRight(diff));
        } else if target_cell < self.current_pointer {
            let diff = self.current_pointer - target_cell;
            self.emit(BFO::MoveLeft(diff));
        }
        self.current_pointer = target_cell;
    }

    fn compile_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Declare { var_type: _, name, value } => {
                // 1. Evaluate expression into a temporary cell
                // As typical C-like, we allocate a new cell for this variable.
                let cell_index = self.next_free_cell;
                self.next_free_cell += 1;
                self.symbol_table.insert(name, cell_index);

                // Compile expr result INTO this cell
                self.compile_expr_into(value, cell_index);
            },
            Stmt::Assign { name, value } => {
                let cell_index = *self.symbol_table.get(&name).expect("Undefined variable");
                // Clear the cell first? Or assume overwrite? 
                // BF doesn't have 'mov', only add/sub. 
                // SAFE: Clear cell then add.
                self.move_to(cell_index);
                self.emit(BFO::Clear);
                self.compile_expr_into(value, cell_index);
            },
            Stmt::Print(expr) => {
                if let Expr::StringLiteral(s) = expr {
                    // Print string literal
                    let temp_cell = self.next_free_cell;
                    self.next_free_cell += 1; // Use a temp cell
                    self.move_to(temp_cell);
                    
                    // We need to be careful with optimization, but simple approach:
                    // clear temp (0), then for each char: set to char, print.
                    self.emit(BFO::Clear); 
                    
                    let mut current_val = 0;
                    for b in s.bytes() {
                        let diff = (b as i16) - current_val;
                        if diff > 0 {
                            self.emit(BFO::Add(diff as u8));
                        } else if diff < 0 {
                            self.emit(BFO::Sub((-diff) as u8));
                        }
                        self.emit(BFO::Print);
                        current_val = b as i16;
                    }
                    // Reset temp to 0 (optional but good practice)
                    self.emit(BFO::Clear); 
                } else {
                    // Calculate expr result into a temporary cell
                    let temp_cell = self.next_free_cell;
                    self.next_free_cell += 1;
                
                    self.compile_expr_into(expr, temp_cell);
                    self.move_to(temp_cell);
                    self.emit(BFO::Print);
                }
            },
            Stmt::While { condition, body } => {
                // WHILE depends on a cell being non-zero.
                // We evaluate condition into a temp cell.
                let cond_cell = self.next_free_cell;
                self.next_free_cell += 1;

                self.compile_expr_into(condition.clone(), cond_cell);
                self.move_to(cond_cell);
                
                // We need to capture the loop body.
                // Caveat: The condition must be re-evaluated at the end of the loop!
                // BF `[` checks current cell. 
                // Structure:
                // Eval Cond -> Temp
                // Move Temp
                // [
                //   Body
                //   Eval Cond -> Temp (Re-evaluate)
                //   Move Temp
                // ]
                
                // But wait, "Body" might move the pointer! 
                // We must ensure we are back at `cond_cell` at start/end of loop block.
                
                // This is tricker in single-pass. 
                // Let's create a sub-compiler for the body?
                
                /* 
                   Correction: The standard BF loop `[` enters if current != 0. 
                   Inside, we execute body. Then we hit `]`. It checks current != 0.
                   So we MUST re-evaluate condition at the end of body and put result in `cond_cell`.
                */

                let mut loop_ops = Vec::new();
                
                // To record instructions for the loop body, we accept that 'emit' pushes to self.instructions.
                // We need to intercept them.
                // Let's temporarily swap `self.instructions`.
                let parent_instructions = std::mem::replace(&mut self.instructions, Vec::new());
                
                // Compile Body
                for s in body {
                    self.compile_stmt(s);
                }
                // Re-evaluate condition into cond_cell
                self.compile_expr_into(condition, cond_cell);
                // Ensure pointer is back at cond_cell for the Loop to check
                self.move_to(cond_cell);

                loop_ops = std::mem::replace(&mut self.instructions, parent_instructions);
                
                self.emit(BFO::Loop(loop_ops));
            }
        }
    }

    fn compile_expr_into(&mut self, expr: Expr, target_cell: usize) {
        match expr {
            Expr::Number(n) => {
                self.move_to(target_cell);
                // We assume cell is 0 (cleared previously) or we clear it?
                // The caller (Let/Assign) might have cleared it. 
                // But `compile_expr_into` contract generally implies "add result to target".
                // If we want "set target to result", we should ensure clear.
                // Let's assume the caller handles clearing if needed, OR we just do adds.
                // Actually, for `Let x = 5`, x is fresh (0). `Assign` clears. 
                // So we just Add.
                if n > 0 {
                    self.emit(BFO::Add(n as u8));
                } else if n < 0 {
                    self.emit(BFO::Sub((-n) as u8));
                }
            },
            Expr::Variable(name) => {
                let src_cell = *self.symbol_table.get(&name).expect("Undefined variable");
                // Copy src_cell to target_cell
                // BF Copy: defaults to destructive move `[-]`. 
                // Non-destructive copy requires a temp cell.
                // Temp1 = 0.
                // Src -> Move to Target & Temp1.
                // Temp1 -> Restore Src.
                
                let temp_cell = self.next_free_cell;
                self.next_free_cell += 1;
                
                // 1. Move to Src
                self.move_to(src_cell);
                
                // 2. Loop to move value to Target and Temp
                // [ - >+ >+ << ]  (Src -> Target, Temp) assuming layout Src, Target, Temp
                // Implementation:
                // [-] with body adding to T and Temp.
                
                // We can't generate raw BF loop easily here without BFO support for "MoveAdd".
                // Let's synthesize the loop manually with BFO ops.
                
                // We are at src_cell.
                let mut loop_body = Vec::new();
                
                // -1 at src
                loop_body.push(BFO::Sub(1));
                
                // +1 at target
                let diff_tgt = if target_cell > src_cell { target_cell - src_cell } else { 0 }; // Handle direction
                 // Simple logic:
                if target_cell > src_cell {
                    loop_body.push(BFO::MoveRight(target_cell - src_cell));
                    loop_body.push(BFO::Add(1));
                    loop_body.push(BFO::MoveLeft(target_cell - src_cell));
                } else {
                     loop_body.push(BFO::MoveLeft(src_cell - target_cell));
                    loop_body.push(BFO::Add(1));
                    loop_body.push(BFO::MoveRight(src_cell - target_cell));
                }

                // +1 at temp
                if temp_cell > src_cell {
                    loop_body.push(BFO::MoveRight(temp_cell - src_cell));
                    loop_body.push(BFO::Add(1));
                    loop_body.push(BFO::MoveLeft(temp_cell - src_cell));
                }
                
                self.emit(BFO::Loop(loop_body));
                
                // 3. Restore Src from Temp
                self.move_to(temp_cell);
                 let mut restore_loop = Vec::new();
                restore_loop.push(BFO::Sub(1));
                // Move back to src and add 1
                // logic similar to above
                 if src_cell < temp_cell {
                    restore_loop.push(BFO::MoveLeft(temp_cell - src_cell));
                    restore_loop.push(BFO::Add(1));
                    restore_loop.push(BFO::MoveRight(temp_cell - src_cell));
                }
                self.emit(BFO::Loop(restore_loop));
                
                // End at temp_cell (which is now 0).
                self.current_pointer = temp_cell;
            },
            Expr::StringLiteral(_) => {
                // Not supported in variable assignment yet (only Print handles string literals directly?)
                // Or we can store string... no, v1 supports strings only in Print?
                // Let's implement strings for Print only for now.
                panic!("Assigning strings to variables not yet supported");
            },
            Expr::BinaryOperation { left, op, right } => {
                // Evaluate left -> target
                self.compile_expr_into(*left, target_cell);
                
                // Evaluate right -> temp
                let temp_cell = self.next_free_cell;
                self.next_free_cell += 1;
                self.compile_expr_into(*right, temp_cell);
                
                // Perform Op (Target = Target op Temp)
                self.move_to(temp_cell);
                
                // Destructive add/sub from Temp to Target
                let mut op_loop = Vec::new();
                op_loop.push(BFO::Sub(1)); // Decrement temp
                
                // Move to target
                if target_cell < temp_cell {
                    op_loop.push(BFO::MoveLeft(temp_cell - target_cell));
                } else {
                     op_loop.push(BFO::MoveRight(target_cell - temp_cell));
                }
                
                match op {
                    Token::Plus => op_loop.push(BFO::Add(1)),
                    Token::Minus => op_loop.push(BFO::Sub(1)),
                    _ => panic!("Unsupported binary op"),
                }
                
                // Move back to temp
                 if target_cell < temp_cell {
                    op_loop.push(BFO::MoveRight(temp_cell - target_cell));
                } else {
                     op_loop.push(BFO::MoveLeft(target_cell - temp_cell));
                }
                
                self.emit(BFO::Loop(op_loop));
                
                // Temp is now 0.
                self.current_pointer = temp_cell;
            }
        }
    }
}
