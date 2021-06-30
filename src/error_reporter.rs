use std::sync::Mutex;
use std::mem;

use crate::token;
use crate::token::Token;

// TODO: this is awful and i hate it
lazy_static! {
    pub static ref ERROR_REPORTER: Mutex<ErrorReporter> = Mutex::new(ErrorReporter::new());
}

pub struct ErrorReporter {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter {
    fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn runtime_error_on_token(&mut self, token: &Token, message: &str) {
        if mem::discriminant(token.token_type()) == mem::discriminant(&token::Type::EOF) {
            self.report_runtime_error(token.line(), " at end", message);
        } else {
            self.report_runtime_error(token.line(), &format!(" at '{}'", token.lexeme()), message)
        }
    }

    pub fn runtime_error(&mut self, message: &str) {
        eprintln!("Runtime Error: {}", message);
        self.had_runtime_error = true;
    }

    fn report_runtime_error(&mut self, line: usize, position: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, position, message);
        self.had_runtime_error = true;
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn error_on_token(&mut self, token: &Token, message: &str) {
        if mem::discriminant(token.token_type()) == mem::discriminant(&token::Type::EOF) {
            self.report(token.line(), " at end", message);
        } else {
            self.report(token.line(), &format!(" at '{}'", token.lexeme()), message)
        }
    }

    fn report(&mut self, line: usize, position: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, position, message);
        self.had_error = true;
    }
}
