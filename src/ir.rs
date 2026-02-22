
#[derive(Debug, Clone)]
pub enum BFO {
    Modify(i16),     // +/- n
    Shift(isize),    // +/- n
    Print,           // .
    Scan,            // ,
    Loop(Vec<BFO>),  // [ ... ]
    
    // Higher level ops that might be lowered later or directly supported
    Clear,           // [-]
    ForceGoto(usize), // Force set pointer location without moves
}
