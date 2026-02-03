
#[derive(Debug, Clone)]
pub enum BFO {
    Add(u8),         // +n
    Sub(u8),         // -n
    MoveRight(usize),// >n
    MoveLeft(usize), // <n
    Print,           // .
    Loop(Vec<BFO>),  // [ ... ]
    
    // Higher level ops that might be lowered later or directly supported
    Clear,           // [-]
}
