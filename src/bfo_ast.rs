// BFO Abstract Syntax Tree
// Represents parsed BFO intermediate format

#[derive(Debug, Clone, PartialEq)]
pub struct BFOProgram {
    pub items: Vec<BFOItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BFOItem {
    // Function definition
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<BFOStmt>,
    },
    
    // Top-level statement
    Statement(BFOStmt),

    // Include another BFO file
    Include(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BFOStmt {
    // Memory Management
    New { name: String, size: usize },     // new x 10
    Free { name: String },                 // free x
    Goto { value: BFOValue },              // goto x or goto x+2
    At(BFOValue),                          // @ x or @ x+2

    // Pointer-Relative Operations
    Shift(isize),                          // shift +/- n
    Alias { name: String, value: BFOValue }, // ref alias_name existing_var

    // Value Operations (at current pointer)
    Modify(i16),                           // modify +/- n
    Set(u8),                               // set 1
    Print,                                 // print
    Scan,                                  // scan

    // Control Flow
    Loop(Vec<BFOStmt>),                    // loop { ... }
    
    // Function Call
    Call { name: String, args: Vec<BFOValue> },

    // { ... } - Block for logical grouping/scoping
    Block(Vec<BFOStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BFOValue {
    Number(i32),
    Char(char),
    Variable(String, usize),
}
