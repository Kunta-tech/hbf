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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BFOStmt {
    // set x value
    Set { name: String, value: BFOValue },
    
    // new x value
    New { name: String, value: BFOValue },
    
    // add x value
    Add { name: String, value: BFOValue },
    
    // sub x value
    Sub { name: String, value: BFOValue },
    
    // print x
    Print { value: BFOValue },
    
    // while x { ... }
    While { condition: String, body: Vec<BFOStmt> },
    
    // function_name(args)
    Call { name: String, args: Vec<BFOValue> },

    // free x
    Free { name: String },

    // { ... }
    Block(Vec<BFOStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BFOValue {
    Number(i32),
    Char(char),
    Variable(String),
}
