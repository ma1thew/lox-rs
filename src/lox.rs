use std::process;
use std::io;
use std::io::Write;
use std::fs;

use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::error_reporter::ERROR_REPORTER;
use crate::statement::FunctionType;
use crate::expression::ClassType;
use crate::util::{EX_DATAERR, EX_SOFTWARE};

// TODO: This reeks of OOP.
pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &str) {
        self.run(&fs::read_to_string(path).expect(&format!("Failed to open source file: {}", path)));
        if ERROR_REPORTER.lock().unwrap().had_error {
            process::exit(EX_DATAERR);
        }
        if ERROR_REPORTER.lock().unwrap().had_runtime_error {
            process::exit(EX_SOFTWARE);
        }
    }

    pub fn run_prompt(&mut self) {
        let mut input_buffer = String::new();
        loop {
            print!("> ");
            io::stdout().flush().expect("Error flushing stdout");
            match io::stdin().read_line(&mut input_buffer) {
                Ok(_) => {
                    if input_buffer.is_empty() {
                        println!("\nBye!");
                        break;
                    }
                    self.run(&input_buffer);
                    ERROR_REPORTER.lock().unwrap().had_error = false;
                },
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                },
            }
            input_buffer.clear();
        }
    }

    fn run(&mut self, source: &str) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let mut statements = parser.parse();

        if ERROR_REPORTER.lock().unwrap().had_error {
            return
        }

        let mut scopes = Vec::new();
        let function_type = FunctionType::None;
        let class_type = ClassType::None;
        for statement in &mut statements {
            statement.resolve(&mut scopes, &function_type, &class_type);
        }
        if ERROR_REPORTER.lock().unwrap().had_error {
            return
        }
        self.interpreter.interpret(statements);
    }
}
