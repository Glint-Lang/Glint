use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::io::{self, Write};

struct Interpreter {
    functions: HashMap<String, Function>,
    variables: HashMap<String, Value>,
    program: Vec<Value>,
    cache: HashMap<String, Value>,
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
            variables: HashMap::new(),
            program: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// ➕ Adds a new function to the functions map
    fn add_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }

    /// ➕ Adds a new variable to the variables map
    fn add_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 🎬 Interprets the loaded program by processing function calls and write statements
    fn interpret(&mut self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        write!(handle, "\n\nFunctions:\n\n").unwrap();
        for (name, func) in self.functions.iter() {
            write!(
                handle,
                "Function:\n  Name: \"{}\"\n  Args: {:?}\n  Body: {}\n\n",
                name,
                func.args,
                serde_json::to_string_pretty(&func.body).unwrap()
            )
            .unwrap();
        }

        let program_len = self.program.len();
        for i in 0..program_len {
            let element = &self.program[i];

            let function_call = element
                .get("FunctionCall")
                .map(|v| v.as_object().unwrap().clone());
            let write_objs = element.get("Write").map(|v| {
                if v.is_array() {
                    v.as_array().unwrap().clone()
                } else {
                    panic!("Expected 'Write' to be an array but got something else.");
                }
            });
            let var_assign = element
                .get("VariableAssign")
                .map(|v| v.as_object().unwrap().clone());

            if let Some(call_obj) = function_call {
                self.process_function_call(&call_obj);
            } else if let Some(write_array) = write_objs {
                // Let's open a line to record the output
                let mut output_line = String::new();
                for write_elem in write_array {
                    if let Some(write_obj) = write_elem.as_object() {
                        // Processing of each element of the Write array
                        output_line.push_str(&self.process_write(write_obj, &HashMap::new()));
                    }
                }
                // We write the line immediately after all the Write elements
                writeln!(handle, "{}", output_line).unwrap();
            } else if let Some(var_assign) = var_assign {
                self.process_variable_assign(&var_assign);
            }
        }
    }

    /// ➕ Processes a variable assignment and adds it to the variables map
    fn process_variable_assign(&mut self, var_assign: &serde_json::Map<String, Value>) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let var_name = var_assign.get("name").unwrap().as_str().unwrap();
        let var_value = var_assign.get("value").unwrap();

        let cache_key = format!("VariableAssign:{:?}", var_assign); // Creating a key for caching

        write!(handle, "{}\n", cache_key).unwrap();

        if let Some(cached_result) = self.cache.get(&cache_key) {
            self.variables
                .insert(var_name.to_string(), cached_result.clone()); // Returning the value from the cache
        } else {
            self.variables
                .insert(var_name.to_string(), var_value.clone());
            self.cache.insert(cache_key, var_value.clone()); // Caching a variable
        }
    }

    /// 🖋️ Handles the Write statement, which can be a string, identifier, integer, function call, or binary operation
    fn process_write(
        &mut self,
        write_obj: &serde_json::Map<String, Value>,
        local_scope: &HashMap<String, Value>,
    ) -> String {
        let cache_key = format!("{:?}", write_obj); // Creating a key for the cache
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.as_str().unwrap().to_string(); // Returning the cached result
        }

        let result = if let Some(binary_op) = write_obj.get("BinaryOp") {
            let result = self.evaluate_binary_op(binary_op, local_scope);
            result.to_string()
        } else if let Some(string_val) = write_obj.get("String") {
            string_val.as_str().unwrap().to_string()
        } else if let Some(identifier) = write_obj.get("Identifier") {
            let id_str = identifier.as_str().unwrap();
            // First, let's check in the local scope (if in the function)
            if let Some(val) = local_scope.get(id_str) {
                let resolved_value = self.extract_value(val);
                match resolved_value {
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s,
                    _ => "Unsupported type".to_string(),
                }
            } else if let Some(global_val) = self.variables.get(id_str) {
                // If it is not found in the local scope, check the global variables
                let resolved_value = self.extract_value(global_val);
                match resolved_value {
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s,
                    _ => "Unsupported type".to_string(),
                }
            } else {
                format!("Identifier '{}' not found", id_str)
            }
        } else if let Some(integer_val) = write_obj.get("Integer") {
            integer_val.as_i64().unwrap().to_string()
        } else if let Some(function_call) = write_obj.get("FunctionCall") {
            let result = self.process_function_call(function_call.as_object().unwrap());
            result.to_string()
        } else {
            "Unknown data type in Write statement".to_string()
        };

        self.cache.insert(cache_key, Value::String(result.clone())); // Caching the result
        result
    }

    /// 📞 Processes a function call and returns its result
    fn process_function_call(&mut self, call_obj: &serde_json::Map<String, Value>) -> i64 {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let cache_key = format!("{:?}", call_obj); // Creating a key for caching
        write!(handle, "{}\n", cache_key).unwrap();

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.as_i64().unwrap(); // We return the result from the cache, if there is one
        }

        if let Some(name) = call_obj.get("name").and_then(Value::as_str) {
            if let Some(func) = self.functions.get(name).cloned() {
                let args = call_obj["args"].as_array().unwrap();
                if args.len() == func.args.len() {
                    let arg_map: HashMap<String, Value> = func
                        .args
                        .iter()
                        .cloned()
                        .zip(args.iter().cloned())
                        .collect();

                    // Creating a new array of variables for the local scope of the function
                    let mut local_scope = self.variables.clone(); // Cloning global variables
                    local_scope.extend(arg_map.clone()); // Adding arguments to the local scope

                    // We execute all commands from the body of the function with a local scope
                    let result = self.execute_function_body(&func.body, &local_scope);

                    self.cache.insert(cache_key, Value::Number(result.into())); // Caching the result
                    return result;
                } else {
                    write!(
                        handle,
                        "Error: Function '{}' expects {} arguments but {} were provided\n",
                        name,
                        func.args.len(),
                        args.len()
                    )
                    .unwrap();
                }
            } else {
                write!(handle, "Function '{}' not found\n", name).unwrap();
            }
        }
        0
    }

    /// 🛠️ Executes the body of a function and returns a result (if any)
    fn execute_function_body(&mut self, body: &Value, local_scope: &HashMap<String, Value>) -> i64 {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let cache_key = format!("{:?}{:?}", body, local_scope);
        write!(handle, "{}\n", cache_key).unwrap();

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.as_i64().unwrap();
        }

        let mut return_value: Option<i64> = None;
        let mut current_scope = local_scope.clone();

        if let Some(block) = body.get("Block").and_then(Value::as_array) {
            for statement in block {
                if let Some(write_array) = statement.get("Write").and_then(Value::as_array) {
                    let mut output_line = String::new();
                    for write_elem in write_array {
                        if let Some(write_obj) = write_elem.as_object() {
                            output_line.push_str(&self.process_write(write_obj, &current_scope));
                        }
                    }
                    writeln!(handle, "{}", output_line).unwrap();
                }
                if let Some(var_assign) = statement.get("VariableAssign").and_then(Value::as_object)
                {
                    // We use the existing process_variable_assign function to process
                    self.process_variable_assign(var_assign);
                    let var_name = var_assign
                        .get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();
                    let var_value = var_assign.get("value").unwrap().clone();
                    current_scope.insert(var_name, var_value);
                }
                if let Some(function_call) =
                    statement.get("FunctionCall").and_then(Value::as_object)
                {
                    // We use the existing process_function_call function to process
                    self.process_function_call(function_call);
                }
                if let Some(return_obj) = statement.get("Return").and_then(Value::as_object) {
                    return_value = Some(self.process_return(return_obj, &current_scope));
                }
            }
        }

        let result = return_value.unwrap_or(0);
        self.cache.insert(cache_key, Value::Number(result.into())); // Caching the result
        result
    }

    /// ↩️ Processes the Return statement and extracts the value to be returned
    fn process_return(
        &mut self,
        return_obj: &serde_json::Map<String, Value>,
        local_scope: &HashMap<String, Value>,
    ) -> i64 {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let cache_key = format!("Return:{:?}{:?}", return_obj, local_scope); // Creating a key for caching

        write!(handle, "{}\n", cache_key).unwrap();

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.as_i64().unwrap(); // Returning the result from the cache
        }

        let result = if let Some(identifier) = return_obj.get("Identifier") {
            if let Some(val) = local_scope.get(identifier.as_str().unwrap()) {
                self.extract_value(val).as_i64().unwrap()
            } else {
                write!(
                    handle,
                    "Return identifier '{}' not found\n",
                    identifier.as_str().unwrap()
                )
                .unwrap();
                0
            }
        } else if let Some(binary_op) = return_obj.get("BinaryOp") {
            self.evaluate_binary_op(binary_op, local_scope)
                .as_i64()
                .unwrap()
        } else {
            write!(handle, "Unknown return type\n").unwrap();
            0
        };

        self.cache.insert(cache_key, Value::Number(result.into())); // Caching the result
        result
    }

    /// ➕ Evaluates a binary operation (e.g., addition, subtraction, multiplication, division)
    fn evaluate_binary_op(
        &mut self,
        binary_op: &Value,
        local_scope: &HashMap<String, Value>,
    ) -> Value {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let left = self.resolve_value(&binary_op["left"], local_scope);
        let right = self.resolve_value(&binary_op["right"], local_scope);
        let op = binary_op["op"].as_str().unwrap();

        let cache_key = format!("{:?} {} {:?}", left, op, right); // Creating a key for the cache
        write!(handle, "{}\n", cache_key).unwrap();

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.clone(); // Returning a value from the cache, if there is one
        }

        let result = match op {
            "+" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int + right_int).into())
                } else {
                    write!(
                        handle,
                        "BinaryOp error: one of the operands is not an integer.\n"
                    )
                    .unwrap();
                    Value::Null
                }
            }
            "-" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int - right_int).into())
                } else {
                    write!(
                        handle,
                        "BinaryOp error: one of the operands is not an integer.\n"
                    )
                    .unwrap();
                    Value::Null
                }
            }
            "*" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    Value::Number((left_int * right_int).into())
                } else {
                    write!(
                        handle,
                        "BinaryOp error: one of the operands is not an integer.\n"
                    )
                    .unwrap();
                    Value::Null
                }
            }
            "/" => {
                if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
                    if right_int != 0 {
                        Value::Number((left_int / right_int).into())
                    } else {
                        write!(handle, "Error: Division by zero\n").unwrap();
                        Value::Null
                    }
                } else {
                    write!(
                        handle,
                        "BinaryOp error: one of the operands is not an integer.\n"
                    )
                    .unwrap();
                    Value::Null
                }
            }
            _ => {
                write!(handle, "Unknown binary operator: {}\n", op).unwrap();
                Value::Null
            }
        };

        self.cache.insert(cache_key, result.clone()); // Saving the result to the cache
        write!(handle, "Caching completed successfully!\n").unwrap();
        result
    }

    /// 🔍 Resolves a value from an identifier, string, integer, or binary operation
    fn resolve_value(&mut self, value: &Value, local_scope: &HashMap<String, Value>) -> Value {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if let Some(identifier) = value.as_object().and_then(|v| v.get("Identifier")) {
            let id_str = identifier.as_str().unwrap();
            if let Some(val) = local_scope.get(id_str) {
                self.extract_value(val) // 🧲 Getting the value of the variable
            } else if let Some(global_val) = self.variables.get(id_str) {
                self.extract_value(global_val) // 🧲 We get the value of the global variable
            } else {
                write!(handle, "Identifier '{}' not found\n", id_str).unwrap();
                Value::Null
            }
        } else if let Some(integer_obj) = value.as_object().and_then(|v| v.get("Integer")) {
            Value::Number(integer_obj.as_i64().unwrap().into()) // 🔢 Extracts and returns the integer directly
        } else if let Some(binary_op) = value.as_object().and_then(|v| v.get("BinaryOp")) {
            self.evaluate_binary_op(binary_op, local_scope) // ➕ Processes and returns the result of a binary operation
        } else {
            write!(handle, "Unexpected value type: {:?}\n", value).unwrap(); // ⚠️ Unexpected type error
            Value::Null
        }
    }

    /// 🧲 Extracts the actual value from a Value type (e.g., Integer, String, or other)
    fn extract_value(&self, value: &Value) -> Value {
        if let Some(integer) = value.get("Integer") {
            Value::Number(integer.as_i64().unwrap().into()) // 🔢 Extracts an integer value
        } else if let Some(string) = value.get("String") {
            Value::String(string.as_str().unwrap().to_string()) // 📝 Extracts a string value
        } else {
            value.clone() // 📝 Returns the value as-is for other types
        }
    }

    /// 📂 Loads the program and functions from a JSON string
    fn load_from_json(&mut self, json_str: &str) {
        let data: Value = from_str(json_str).unwrap();
        self.program = data["Program"].as_array().unwrap().to_vec();

        let functions: Vec<Function> = self
            .program
            .iter()
            .filter_map(|element| {
                element.get("Function").map(|func_obj| Function {
                    name: func_obj["name"].as_str().unwrap().to_string(),
                    args: func_obj["args"]["FunctionArgs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|arg| arg["Identifier"].as_str().unwrap().to_string())
                        .collect(),
                    body: func_obj["body"].clone(),
                })
            })
            .collect();

        for func in functions {
            self.add_function(func);
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        write!(handle, "Variables:\n\n").unwrap();
        for element in &self.program {
            if let Some(var_assign) = element.get("VariableAssign") {
                let var_assign_obj = var_assign.as_object().unwrap();
                let name = var_assign_obj.get("name").unwrap().as_str().unwrap();
                let value = var_assign_obj.get("value").unwrap();
                write!(
                    handle,
                    "Variable:\n  Name: \"{}\"\n  Value: {}\n\n",
                    name,
                    serde_json::to_string_pretty(value).unwrap()
                )
                .unwrap();
            }
        }
    }
}

/// 🎬 Entry point: Initializes the interpreter and runs the program from a JSON string
pub fn interpret_from_json(json_str: &str) {
    let mut interpreter = Interpreter::new();
    interpreter.load_from_json(json_str); // 📂 Loads the program from JSON
    interpreter.interpret(); // 🎬 Interprets and executes the program
}
