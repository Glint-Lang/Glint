use crate::ast::AST;
use std::collections::HashMap;

pub struct Interpreter {
    variables: HashMap<String, AST>,
    functions: HashMap<String, (Vec<String>, AST)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, ast: AST) {
        match ast {
            AST::Program(statements) => {
                for statement in statements {
                    self.interpret(statement);
                }
            }
            AST::Write(expr) => {
                let evaluated_exprs = if let AST::Array(exprs) = *expr {
                    exprs.into_iter().map(|e| self.evaluate(e)).collect::<Vec<_>>()
                } else {
                    vec![self.evaluate(*expr)]
                };

                // Объединение всех значений в одну строку
                let mut output = String::new();
                for value in evaluated_exprs {
                    output.push_str(&self.format_value(value));
                }
                // Вывод объединенной строки
                println!("{}", output);
            }
            AST::VariableAssign { name, value } => {
                let evaluated_value = self.evaluate(*value);
                self.variables.insert(name, evaluated_value);
            }
            AST::Return(expr) => {
                let value = self.evaluate(*expr);
                self.variables.insert("return".to_string(), value);
            }
            AST::BinaryOp { left, op, right } => {
                let left_val = self.evaluate(*left);
                let right_val = self.evaluate(*right);
                self.apply_binary_op(left_val, op, right_val);
            }
            AST::Identifier(name) => {
                if let Some(value) = self.variables.get(&name) {
                    println!("{}", self.format_value(value.clone()));
                } else {
                    eprintln!("Undefined variable: {}", name);
                }
            }
            AST::FunctionCall { name, args } => {
                if let Some((params, body)) = self.functions.get(&name).cloned() {
                    let mut interpreter = Interpreter::new();
                    for (param, arg) in params.iter().zip(args) {
                        let evaluated_arg = self.evaluate(arg);
                        interpreter.variables.insert(param.clone(), evaluated_arg);
                    }
                    interpreter.interpret(body.clone());
                    if let Some(return_value) = interpreter.variables.get("return") {
                        self.variables.insert("return".to_string(), return_value.clone());
                    }
                } else {
                    eprintln!("Undefined function: {}", name);
                }
            }
            _ => {
                eprintln!("Unsupported AST node: {:?}", ast);
            }
        }
    }

    fn evaluate(&mut self, ast: AST) -> AST {
        match ast {
            AST::Identifier(name) => {
                if let Some(value) = self.variables.get(&name) {
                    value.clone()
                } else {
                    eprintln!("Undefined variable: {}", name);
                    AST::Identifier(name)
                }
            }
            AST::BinaryOp { left, op, right } => {
                let left_val = self.evaluate(*left);
                let right_val = self.evaluate(*right);
                self.apply_binary_op(left_val, op, right_val)
            }
            AST::FunctionCall { name, args } => {
                if let Some((params, body)) = self.functions.get(&name).cloned() {
                    let mut new_scope = Interpreter::new();
                    for (param, arg) in params.iter().zip(args) {
                        let evaluated_arg = self.evaluate(arg);
                        new_scope.variables.insert(param.clone(), evaluated_arg);
                    }
                    new_scope.interpret(body.clone());
                    if let Some(return_value) = new_scope.variables.get("return") {
                        return_value.clone()
                    } else {
                        AST::Bool(false)
                    }
                } else {
                    eprintln!("Undefined function: {}", name);
                    AST::Bool(false)
                }
            }
            _ => ast,
        }
    }

    fn apply_binary_op(&self, left: AST, op: String, right: AST) -> AST {
        match op.as_str() {
            "+" | "-" | "*" | "/" => {
                match (self.coerce_to_float(left.clone()), self.coerce_to_float(right.clone())) {
                    (AST::Float(l), AST::Float(r)) => match op.as_str() {
                        "+" => AST::Float(l + r),
                        "-" => AST::Float(l - r),
                        "*" => AST::Float(l * r),
                        "/" => AST::Float(l / r),
                        _ => AST::Bool(false),  // This should never happen
                    },
                    _ => AST::Bool(false),  // Error if unable to coerce types
                }
            }
            _ => {
                eprintln!("Unsupported binary operation: {:?} {} {:?}", left, op, right);
                AST::Bool(false)
            }
        }
    }

    fn coerce_to_float(&self, value: AST) -> AST {
        match value {
            AST::Integer(i) => AST::Float(i as f64),
            AST::Float(f) => AST::Float(f),
            _ => value,
        }
    }

    fn format_value(&self, ast: AST) -> String {
        match ast {
            AST::Integer(i) => i.to_string(),
            AST::Float(f) => f.to_string(),
            AST::Bool(b) => b.to_string(),
            AST::String(s) => s,
            AST::Array(elements) => {
                let formatted_elements: Vec<String> = elements.into_iter().map(|e| self.format_value(e)).collect();
                format!("[{}]", formatted_elements.join(","))
            }
            AST::Dictionary(pairs) => {
                let formatted_pairs: Vec<String> = pairs.into_iter()
                    .map(|(key, value)| format!("{}: {}", self.format_value(key), self.format_value(value)))
                    .collect();
                format!("{{{}}}", formatted_pairs.join(", "))
            }
            AST::Tuple(elements) => {
                let formatted_elements: Vec<String> = elements.into_iter().map(|e| self.format_value(e)).collect();
                format!("({})", formatted_elements.join(", "))
            }
            _ => "Unsupported value".to_string(),
        }
    }
}

