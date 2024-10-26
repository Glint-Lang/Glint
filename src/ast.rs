use serde::{Serialize, Deserialize};

// 🧩 Represents the Abstract Syntax Tree (AST)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AST {
    // 📝 A program consisting of multiple AST nodes
    Program(Vec<AST>),

    // 🛠️ A function with a name, arguments, and a body
    Function {
        name: String,
        args: Box<AST>,
        body: Box<AST>,
    },

    // 📞 A function call with a name and arguments
    FunctionCall {
        name: String,
        args: Vec<AST>,
    },

    // 🔙 A return statement with an expression
    Return(Box<AST>),

    // ✍️ A write operation with a list of expressions
    Write(Vec<AST>),


    // ➕ A binary operation with left operand, operator, and right operand
    BinaryOp {
        left: Box<AST>,
        op: String,
        right: Box<AST>,
    },

    // 🔤 An identifier (variable or function name)
    Identifier(String),

    // 🔢 An integer literal
    Integer(i32),

    // 🔣 A floating-point literal
    Float(f64),

    // ✅ A boolean value (true or false)
    Bool(bool),

    // 📝 A string literal
    String(String),

    // 📚 An array containing multiple AST nodes
    Array(Vec<AST>),

    // 📖 A dictionary (key-value pairs)
    Dictionary(Vec<(AST, AST)>),

    // 🎭 A tuple containing multiple AST nodes
    Tuple(Vec<AST>),

    // 🛠️ Variable assignment with a name and value
    VariableAssign {
        name: String,
        value: Box<AST>,
    },

    // 🎯 A switch-like expression with cases and an optional default case
    Coincide {
        expr: Box<AST>,
        cases: Vec<(AST, AST)>,
        default: Option<Box<AST>>,
    },

    // 🧱 A block of multiple AST nodes
    Block(Vec<AST>),

    // 🧠 If-Else statement with condition, if-block, and optional else-block
    IfElse {
        condition: Box<AST>,
        if_block: Box<AST>,
        else_block: Option<Box<AST>>,
    },

    // 📋 A list of function arguments
    FunctionArgs(Vec<AST>)
}
