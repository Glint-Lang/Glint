use std::env;
use std::fs;
use std::time::Instant;
use peak_alloc::PeakAlloc;
use Glint::ast::AST;
use Glint::error::ParseError;
use Glint::parser::parse_program;
use Glint::interpreter::Interpreter;
use sysinfo::System;
use colored::Colorize;
use serde_json; // Add this import

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

const INFO: &str = r#"
                 ✧Glint v0.0.1✧
       Usage: Glint [command] [options]
       Commands:
        run <filename>.glt    Run the script
        info                  Display info
       flags:
        -dev                  Display dev info
"#;

fn print_version_info() {
    let header = "✧Glint v0.0.1✧".bright_blue();
    let usage = "Usage:".cyan();
    let commands = "Commands:".cyan();
    let flags = "flags:".cyan();

    let info_colored = INFO
        .replace("✧Glint v0.0.1✧", &header.to_string())
        .replace("Usage:", &usage.to_string())
        .replace("Commands:", &commands.to_string())
        .replace("flags:", &flags.to_string());

    println!("{}", info_colored);
}

fn print_dev_info(start_time: Instant) {
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let peak_mem_gb = PEAK_ALLOC.peak_usage_as_gb();
    let current_mem_mb = PEAK_ALLOC.current_usage_as_mb();

    println!("{} Dev Info {}", "<=>".blue(), "<=>".blue());
    println!("{}: {:.4}s", "Elapsed time".truecolor(41, 176, 255), elapsed_secs);
    println!("{}:", "Resource consumption".truecolor(0, 76, 120));
    println!("  └─ {}: {:.4} MB", "RAM Usage".truecolor(41, 176, 255), current_mem_mb);
    println!("  └─ {}: {:.4} GB", "Peak RAM Usage".truecolor(41, 176, 255), peak_mem_gb);
    println!("  └─ {}: {:?}", "OS".truecolor(41, 176, 255), System::os_version());
    println!("{} End Dev Info {}", "<=>".blue(), "<=>".blue());
}

fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No command provided, print version info
            print_version_info();
        }
        2 => {
            if args[1] == "info" {
                // Print version info
                print_version_info();
            } else {
                eprintln!("Usage: Glint [command] [options]");
            }
        }
        3 | 4 => {
            if args[1] == "run" {
                let filename = &args[2];
                let mut dev_flag = false;

                if args.len() == 4 && args[3] == "-dev" {
                    dev_flag = true;
                }

                // Print developer info before processing the file if -dev is present
                if dev_flag {
                    print_dev_info(start_time);
                }

                let input = match fs::read_to_string(filename) {
                    Ok(contents) => contents,
                    Err(err) => {
                        eprintln!("Error reading file: {}", err);
                        return;
                    }
                };

                match parse_program(&input) {
                    Ok(ast) => {
                        // Serialize the AST to a JSON string
                        let ast_json = serde_json::to_string_pretty(&ast).expect("Failed to serialize AST");
                        println!("Deserialized AST:");
                        println!("{}", ast_json);

                        // Execute the AST using the interpreter
                        let mut interpreter = Interpreter::new();
                        interpreter.interpret(ast);
                    }
                    Err(ParseError::UnknownToken { token, line }) => {
                        eprintln!("Unknown token '{}' on line {}", token, line);
                    }
                    Err(ParseError::IoError(err)) => {
                        eprintln!("IO Error: {}", err);
                    }
                    Err(ParseError::SyntaxError { message, line }) => {
                        eprintln!("Syntax error on line {}: {}", line, message);
                    }
                    Err(ParseError::NomError(_)) => {
                        eprintln!("Parsing error occurred.");
                    }
                }
            } else {
                eprintln!("Usage: Glint run <filename>.glt [-dev]");
            }
        }
        _ => {
            eprintln!("Usage: Glint [command] [options]");
        }
    }
}
