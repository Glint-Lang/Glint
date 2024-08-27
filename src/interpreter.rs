use serde_json::{from_str, Value};
use std::collections::HashMap;

struct Interpreter {
    functions: HashMap<String, Function>,
    program: Vec<Value>,
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    args: Vec<String>,
    body: Value,
}

impl Interpreter {
    /// 🆕 Initializes a new Interpreter with an empty function map and program list
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            program: Vec::new(),
        }
    }

    /// ➕ Adds a new function to the functions map
    fn add_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }

    /// 🎬 Interprets the loaded program by processing function calls and write statements
    fn interpret(&self) {
        // println!("Functions:");
        // for func in self.functions.values() {
        //     println!("{:?}", func);
        // }

        // println!("\nProgram output:");
        for element in &self.program {
            match element.get("FunctionCall") {
                Some(call_obj) => {
                    // 🎯 Processes a function call, but does not output the result
                    self.process_function_call(call_obj.as_object().unwrap());
                }
                None => {
                    if let Some(write_obj) = element.get("Write") {
                        // ✍️ Processes a write statement to output text or a value
                        self.process_write(write_obj.as_object().unwrap(), &HashMap::new());
                    }
                }
            }
        }
    }

    /// 🖋️ Handles the Write statement, which can be a string, identifier, integer, or function call
    fn process_write(&self, write_obj: &serde_json::Map<String, Value>, arg_map: &HashMap<String, Value>) {
        if let Some(binary_op) = write_obj.get("BinaryOp") {
            // If the Write statement contains a BinaryOp, evaluate it
            let result = self.evaluate_binary_op(binary_op, arg_map);
            println!("{}", result.as_i64().unwrap()); // Output the result of the binary operation
        } else if let Some(string_val) = write_obj.get("String") {
            println!("{}", string_val.as_str().unwrap()); // Output a string
        } else if let Some(identifier) = write_obj.get("Identifier") {
            let id_str = identifier.as_str().unwrap();
            if let Some(val) = arg_map.get(id_str) {
                println!("{}", self.extract_value(val).as_i64().unwrap()); // Resolve and print the value of an identifier
            } else {
                println!("Identifier '{}' not found", id_str); // Identifier not found
            }
        } else if let Some(integer_val) = write_obj.get("Integer") {
            println!("{}", integer_val.as_i64().unwrap()); // Output an integer value
        } else if let Some(call_obj) = write_obj.get("FunctionCall") {
            // Executes a function and outputs its result
            let result = self.process_function_call(call_obj.as_object().unwrap());
            println!("{}", result);
        } else {
            println!("Unknown data type in Write statement"); // Unrecognized write statement type
        }
    }


    /// 📞 Processes a function call and returns its result
    fn process_function_call(&self, call_obj: &serde_json::Map<String, Value>) -> i64 {
        if let Some(name) = call_obj.get("name").and_then(Value::as_str) {
            if let Some(func) = self.functions.get(name) {
                let args = call_obj["args"].as_array().unwrap();
                if args.len() == func.args.len() {
                    let arg_map: HashMap<String, Value> = func.args.iter().cloned()
                        .zip(args.iter().cloned()).collect();
                    return self.execute_function_body(&func.body, &arg_map); // 🛠️ Executes the function body
                } else {
                    println!("Error: Function '{}' expects {} arguments but {} were provided", name, func.args.len(), args.len()); // ⚠️ Argument mismatch error
                }
            } else {
                println!("Function '{}' not found", name); // 🔍 Function not found in the map
            }
        }
        0 // 🅾️ Returns zero if the function call could not be processed
    }

    /// 🛠️ Executes the body of a function and returns a result (if any)
    fn execute_function_body(&self, body: &Value, arg_map: &HashMap<String, Value>) -> i64 {
        let mut return_value: Option<i64> = None;

        if let Some(block) = body.get("Block").and_then(Value::as_array) {
            for statement in block {
                if let Some(write_obj) = statement.get("Write") {
                    self.process_write(write_obj.as_object().unwrap(), arg_map); // 🖋️ Processes write statements in the function body
                } else if let Some(return_obj) = statement.get("Return") {
                    return_value = Some(self.process_return(return_obj.as_object().unwrap(), arg_map)); // ↩️ Processes return statements
                }
            }
        }

        return_value.unwrap_or(0) // Returns the result or defaults to zero if no return statement was found
    }

    /// ↩️ Processes the Return statement and extracts the value to be returned
    fn process_return(&self, return_obj: &serde_json::Map<String, Value>, arg_map: &HashMap<String, Value>) -> i64 {
        if let Some(identifier) = return_obj.get("Identifier") {
            if let Some(val) = arg_map.get(identifier.as_str().unwrap()) {
                self.extract_value(val).as_i64().unwrap() // 🧲 Resolves the identifier and returns its value
            } else {
                println!("Return identifier '{}' not found", identifier.as_str().unwrap()); // 🚫 Return identifier not found
                0
            }
        } else if let Some(binary_op) = return_obj.get("BinaryOp") {
            self.evaluate_binary_op(binary_op, arg_map).as_i64().unwrap() // ➕ Evaluates a binary operation and returns the result
        } else {
            println!("Unknown return type"); // ❓ Unrecognized return type
            0
        }
    }

    /// ➕ Evaluates a binary operation (e.g., addition, subtraction, multiplication, division)
    fn evaluate_binary_op(&self, binary_op: &Value, arg_map: &HashMap<String, Value>) -> Value {
        let left = self.resolve_value(&binary_op["left"], arg_map);
        let right = self.resolve_value(&binary_op["right"], arg_map);
        let op = binary_op["op"].as_str().unwrap();

        match op {
            "+" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int + right_int).into()) // ➕ Adds two integers
                } else {
                    println!("BinaryOp error: one of the operands is not an integer."); // ⚠️ Operand type error
                    Value::Null
                }
            }
            "-" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int - right_int).into()) // ➖ Subtracts two integers
                } else {
                    println!("BinaryOp error: one of the operands is not an integer."); // ⚠️ Operand type error
                    Value::Null
                }
            }
            "*" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int * right_int).into()) // ✖️ Multiplies two integers
                } else {
                    println!("BinaryOp error: one of the operands is not an integer."); // ⚠️ Operand type error
                    Value::Null
                }
            }
            "/" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    if right_int != 0 {
                        Value::Number((left_int / right_int).into()) // ➗ Divides two integers
                    } else {
                        println!("Error: Division by zero"); // 🚫 Division by zero error
                        Value::Null
                    }
                } else {
                    println!("BinaryOp error: one of the operands is not an integer."); // ⚠️ Operand type error
                    Value::Null
                }
            }
            _ => {
                println!("Unknown binary operator: {}", op); // ❓ Unrecognized binary operator
                Value::Null
            }
        }
    }

    /// 🔍 Resolves a value from an identifier, string, integer, or binary operation
    fn resolve_value(&self, value: &Value, arg_map: &HashMap<String, Value>) -> Value {
        if let Some(identifier) = value.as_object().and_then(|v| v.get("Identifier")) {
            if let Some(val) = arg_map.get(identifier.as_str().unwrap()) {
                self.extract_value(val) // 🧲 Resolves the identifier to its actual value
            } else {
                println!("Identifier '{}' not found", identifier.as_str().unwrap()); // 🚫 Identifier not found
                Value::Null
            }
        } else if let Some(integer_obj) = value.as_object().and_then(|v| v.get("Integer")) {
            Value::Number(integer_obj.as_i64().unwrap().into()) // 🔢 Extracts and returns the integer directly
        } else if let Some(binary_op) = value.as_object().and_then(|v| v.get("BinaryOp")) {
            self.evaluate_binary_op(binary_op, arg_map) // ➕ Processes and returns the result of a binary operation
        } else {
            println!("Unexpected value type: {:?}", value); // ⚠️ Unexpected type error
            Value::Null
        }
    }

    /// 🧲 Extracts the actual value from a Value type (e.g., Integer, String)
    fn extract_value(&self, value: &Value) -> Value {
        if let Some(integer) = value.get("Integer") {
            Value::Number(integer.as_i64().unwrap().into()) // 🔢 Extracts an integer value
        } else if let Some(string) = value.get("String") {
            Value::String(string.as_str().unwrap().to_string()) // 📝 Extracts a string value
        } else {
            value.clone() // 📝 Returns the value as-is if it's neither an integer nor a string
        }
    }

    /// 📂 Loads the program and functions from a JSON string
    fn load_from_json(&mut self, json_str: &str) {
        let data: Value = from_str(json_str).unwrap();
        self.program = data["Program"].as_array().unwrap().to_vec(); // 📦 Loads the program array from JSON

        let functions: Vec<Function> = self.program.iter()
            .filter_map(|element| element.get("Function").map(|func_obj| {
                Function {
                    name: func_obj["name"].as_str().unwrap().to_string(), // 🔍 Extracts the function name
                    args: func_obj["args"]["FunctionArgs"].as_array().unwrap().iter()
                        .map(|arg| arg["Identifier"].as_str().unwrap().to_string())
                        .collect(), // 📝 Collects the function arguments
                    body: func_obj["body"].clone(), // 📦 Copies the function body
                }
            }))
            .collect();

        for func in functions {
            self.add_function(func); // ➕ Adds each function to the interpreter
        }
    }
}

/// 🎬 Entry point: Initializes the interpreter and runs the program from a JSON string
pub fn interpret_from_json(json_str: &str) {
    let mut interpreter = Interpreter::new();
    interpreter.load_from_json(json_str); // 📂 Loads the program from JSON
    interpreter.interpret(); // 🎬 Interprets and executes the program
}