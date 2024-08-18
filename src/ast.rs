use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AST {
    Program(Vec<AST>),
    Function {
        name: String,
        args: Box<AST>,
        body: Box<AST>,
    },
    FunctionCall {
        name: String,
        args: Vec<AST>,
    },
    Return(Box<AST>),
    Write(Box<AST>),
    BinaryOp {
        left: Box<AST>,
        op: String,
        right: Box<AST>,
    },
    Identifier(String),
    Integer(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<AST>),
    Dictionary(Vec<(AST, AST)>),
    Tuple(Vec<AST>),
    VariableAssign {
        name: String,
        value: Box<AST>,
    },
    IfElse {
        condition: Box<AST>,
        then_branch: Box<AST>,
        elif_branches: Vec<(AST, AST)>,
        else_branch: Option<Box<AST>>,
    },
    Coincide {
        expr: Box<AST>,
        cases: Vec<(AST, AST)>,
        default: Option<Box<AST>>,
    },
    Block(Vec<AST>),
    FunctionArgs(Vec<AST>)
}