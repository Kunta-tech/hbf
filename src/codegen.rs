
use crate::ir::BFO;

pub struct Codegen {
    output: String,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output: String::new(),
        }
    }

    pub fn generate(&mut self, instructions: &[BFO]) -> String {
        for op in instructions {
            self.emit_op(op);
        }
        self.output.clone()
    }

    fn emit_op(&mut self, op: &BFO) {
        match op {
            BFO::Add(n) => {
                for _ in 0..*n { self.output.push('+'); }
            },
            BFO::Sub(n) => {
                for _ in 0..*n { self.output.push('-'); }
            },
            BFO::MoveRight(n) => {
                for _ in 0..*n { self.output.push('>'); }
            },
            BFO::MoveLeft(n) => {
                for _ in 0..*n { self.output.push('<'); }
            },
            BFO::Print => {
                self.output.push('.');
            },
            BFO::Clear => {
                self.output.push_str("[-]");
            },
            BFO::Loop(body) => {
                self.output.push('[');
                for sub_op in body {
                    self.emit_op(sub_op);
                }
                self.output.push(']');
            }
        }
    }
}
