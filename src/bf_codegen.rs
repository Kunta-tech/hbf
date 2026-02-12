
use crate::ir::BFO;
use std::collections::HashSet;

pub struct Codegen {
    output: String,
    pointer_offset: isize,
    dirty_cells: HashSet<isize>,
    unknown_state: bool,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output: String::new(),
            pointer_offset: 0,
            dirty_cells: HashSet::new(),
            unknown_state: false,
        }
    }

    pub fn generate(&mut self, instructions: &[BFO]) -> String {
        for op in instructions {
            self.emit_op(op);
        }
        self.output.clone()
    }

    fn emit_char(&mut self, c: char) {
        if let Some(last) = self.output.chars().last() {
            if (last == '>' && c == '<') || (last == '<' && c == '>') ||
               (last == '+' && c == '-') || (last == '-' && c == '+') {
                self.output.pop();
                return;
            }
        }
        self.output.push(c);
    }

    fn emit_op(&mut self, op: &BFO) {
        match op {
            BFO::Add(n) => {
                for _ in 0..*n { self.emit_char('+'); }
                self.dirty_cells.insert(self.pointer_offset);
            },
            BFO::Sub(n) => {
                for _ in 0..*n { self.emit_char('-'); }
                self.dirty_cells.insert(self.pointer_offset);
            },
            BFO::MoveRight(n) => {
                for _ in 0..*n { self.emit_char('>'); }
                self.pointer_offset += *n as isize;
            },
            BFO::MoveLeft(n) => {
                for _ in 0..*n { self.emit_char('<'); }
                self.pointer_offset -= *n as isize;
            },
            BFO::Print => {
                self.output.push_str(".\n");
            },
            BFO::Scan => {
                self.output.push_str(",\n");
                self.dirty_cells.insert(self.pointer_offset);
            },
            BFO::Clear => {
                if !self.unknown_state && !self.dirty_cells.contains(&self.pointer_offset) {
                    // Skip redundant clear
                } else {
                    self.output.push_str("\n[-]\n");
                    self.dirty_cells.remove(&self.pointer_offset);
                }
            },
            BFO::Loop(body) => {
                let loop_analysis = analyze_loop(body);
                
                if let Some((0, touched)) = loop_analysis {
                    // Balanced loop
                    if !self.unknown_state {
                        self.output.push_str("\n[");
                        
                        // Enter loop context
                        let prev_unknown = self.unknown_state;
                        self.unknown_state = true;
                        
                        for sub_op in body {
                            self.emit_op(sub_op);
                        }
                        
                        self.output.push_str("]\n");
                        
                        // Restore state
                        self.unknown_state = prev_unknown;
                        
                        // Loop exit guarantees current cell is 0
                        self.dirty_cells.remove(&self.pointer_offset);
                        
                        // Mark other touched cells as dirty
                        for t in touched {
                            if t != 0 {
                                self.dirty_cells.insert(self.pointer_offset + t);
                            }
                        }
                    } else {
                         // Already in unknown state
                         self.output.push_str("\n[");
                         for sub_op in body {
                             self.emit_op(sub_op);
                         }
                         self.output.push_str("]\n");
                    }
                } else {
                    // Unbalanced or complex loop
                    self.unknown_state = true;
                    self.output.push_str("\n[");
                    for sub_op in body {
                        self.emit_op(sub_op);
                    }
                    self.output.push_str("]\n");
                }
            }
        }
    }
}

fn analyze_loop(body: &[BFO]) -> Option<(isize, HashSet<isize>)> {
    let mut current_offset = 0isize;
    let mut touched = HashSet::new();

    for op in body {
        match op {
            BFO::Add(_) | BFO::Sub(_) | BFO::Print | BFO::Scan | BFO::Clear => {
                touched.insert(current_offset);
            }
            BFO::MoveRight(n) => {
                current_offset += *n as isize;
            }
            BFO::MoveLeft(n) => {
                current_offset -= *n as isize;
            }
            BFO::Loop(inner_body) => {
                if let Some((inner_net, inner_touched)) = analyze_loop(inner_body) {
                    if inner_net != 0 {
                        return None;
                    }
                    for t in inner_touched {
                        touched.insert(current_offset + t);
                    }
                } else {
                    return None;
                }
            }
        }
    }

    if current_offset == 0 {
        Some((0, touched))
    } else {
        None
    }
}
