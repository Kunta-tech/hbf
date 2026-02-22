
use crate::ir::BFO;
use std::collections::HashSet;

pub struct Codegen {
    output: String,
    pointer_offset: isize,
    dirty_cells: HashSet<isize>,
    unknown_state: bool,
    allow_cancel: bool,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output: String::new(),
            pointer_offset: 0,
            dirty_cells: HashSet::new(),
            unknown_state: false,
            allow_cancel: true,
        }
    }

    pub fn generate(&mut self, instructions: &[BFO]) -> String {
        for op in instructions {
            self.emit_op(op);
        }
        self.output.clone()
    }

    fn emit_char(&mut self, c: char) {
        if self.allow_cancel {
            // Peek back past any whitespace to see if we can cancel
            let mut rev_chars = self.output.chars().rev().enumerate();
            let mut to_pop = 0;
            let mut found_cancel = false;

            while let Some((i, prev)) = rev_chars.next() {
                if prev.is_whitespace() {
                    continue;
                }
                if (prev == '>' && c == '<') || (prev == '<' && c == '>') ||
                   (prev == '+' && c == '-') || (prev == '-' && c == '+') {
                    to_pop = i + 1;
                    found_cancel = true;
                }
                break;
            }

            if found_cancel {
                for _ in 0..to_pop {
                    self.output.pop();
                }
                return;
            }
        }
        self.output.push(c);
        self.allow_cancel = true;
    }

    fn emit_op(&mut self, op: &BFO) {
        match op {
            BFO::Modify(n) => {
                let abs_n = n.unsigned_abs() as u8;
                if *n > 0 {
                    for _ in 0..abs_n { self.emit_char('+'); }
                } else if *n < 0 {
                    for _ in 0..abs_n { self.emit_char('-'); }
                }
                self.dirty_cells.insert(self.pointer_offset);
            },
            BFO::Shift(n) => {
                let abs_n = n.unsigned_abs();
                if *n > 0 {
                    for _ in 0..abs_n { self.emit_char('>'); }
                } else if *n < 0 {
                    for _ in 0..abs_n { self.emit_char('<'); }
                }
                self.pointer_offset += *n;
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
            BFO::ForceGoto(n) => {
                self.pointer_offset = *n as isize;
                self.unknown_state = false; // We know exactly where we are now
                self.allow_cancel = false;
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
                
                // Every loop exit guarantees current cell is 0
                self.dirty_cells.remove(&self.pointer_offset);
            }
        }
    }
}

fn analyze_loop(body: &[BFO]) -> Option<(isize, HashSet<isize>)> {
    let mut current_offset = 0isize;
    let mut touched = HashSet::new();

    for op in body {
        match op {
            BFO::Modify(_) | BFO::Print | BFO::Scan | BFO::Clear => {
                touched.insert(current_offset);
            }
            BFO::Shift(n) => {
                current_offset += *n;
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
            BFO::ForceGoto(_) => return None,
        }
    }

    if current_offset == 0 {
        Some((0, touched))
    } else {
        None
    }
}
