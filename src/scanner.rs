use std::collections::HashMap;

use crate::token;
use crate::token::Token;
use crate::error_reporter::ERROR_REPORTER;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, token::Type> = {
        let mut m = HashMap::new();
        m.insert("and", token::Type::And);
        m.insert("class", token::Type::Class);
        m.insert("else", token::Type::Else);
        m.insert("false", token::Type::False);
        m.insert("for", token::Type::For);
        m.insert("fun", token::Type::Fun);
        m.insert("if", token::Type::If);
        m.insert("nil", token::Type::Nil);
        m.insert("or", token::Type::Or);
        m.insert("print", token::Type::Print);
        m.insert("return", token::Type::Return);
        m.insert("super", token::Type::Super);
        m.insert("this", token::Type::This);
        m.insert("true", token::Type::True);
        m.insert("var", token::Type::Var);
        m.insert("while", token::Type::While);
        m
    };
}

/*
* NOTE: The scanner operates on unicode scalar values; which is not necessarily what a person
* might think a character is. Regardless, strings are UTF-8 encoded, which means that fancy
* multi-byte stuff is only going to happen in user-defined literals. This should mean that
* there is no functional difference between this approach and splitting on grapheme clusters.
*/
// TODO: use an iterator over chars rather than a Vec<char> here.
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    // TODO: self.tokens is not trivial to clone; we should avoid it here.
    // we consume the scanner here; maybe we can keep this in the future.
    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        
        self.tokens.push(Token::new(token::Type::EOF, String::new(), self.line));
        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(token::Type::LeftParen),
            ')' => self.add_token(token::Type::RightParen),
            '{' => self.add_token(token::Type::LeftBrace),
            '}' => self.add_token(token::Type::RightBrace),
            ',' => self.add_token(token::Type::Comma),
            '.' => self.add_token(token::Type::Dot),
            '-' => self.add_token(token::Type::Minus),
            '+' => self.add_token(token::Type::Plus),
            ';' => self.add_token(token::Type::Semicolon),
            '*' => self.add_token(token::Type::Star),
            '!' => {
                let token = if self.match_next('=') { token::Type::BangEqual } else { token::Type::Bang };
                self.add_token(token);
            },
            '=' => {
                let token = if self.match_next('=') { token::Type::EqualEqual } else { token::Type::Equal };
                self.add_token(token);
            },
            '<' => {
                let token = if self.match_next('=') { token::Type::LessEqual } else { token::Type::Less };
                self.add_token(token);
            },
            '>' => {
                let token = if self.match_next('=') { token::Type::GreaterEqual } else { token::Type::Greater };
                self.add_token(token);
            },
            '/' => {
                if self.match_next('/') {
                    while *self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(token::Type::Slash);
                }
            },
            ' ' | '\r' | '\t' => {},
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    ERROR_REPORTER.lock().unwrap().error(self.line, "Unexpected character.");
                }
            },
        }
    }

    fn advance(&mut self) -> &char {
        self.current += 1;
        self.source.get(self.current - 1).unwrap()
    }

    fn add_token(&mut self, token_type: token::Type) {
        self.tokens.push(Token::new(token_type, self.source[self.start..self.current].iter().collect(), self.line));
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || *self.source.get(self.current).unwrap() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> &char {
        self.source.get(self.current).unwrap_or(&'\0')
    }

    fn peek_next(&self) -> &char {
        self.source.get(self.current + 1).unwrap_or(&'\0')
    }

    fn string(&mut self) {
        while *self.peek() != '"' && !self.is_at_end() {
            if *self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            ERROR_REPORTER.lock().unwrap().error(self.line, "Unterminated string.");
        }

        // Capture closing "
        self.advance();
        // Trim enclosing "
        self.add_token(token::Type::String(self.source[(self.start + 1)..(self.current - 1)].iter().collect()));
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }
        if *self.peek() == '.' && self.peek_next().is_digit(10) {
            // consume the .
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }
        self.add_token(token::Type::Number(self.source[self.start..self.current].iter().collect::<String>().parse::<f64>().unwrap()));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphabetic() || self.peek().is_digit(10) {
            self.advance();
        }
        self.add_token(KEYWORDS.get(&*self.source[self.start..self.current].iter().collect::<String>()).unwrap_or(&token::Type::Identifier).clone());
    }
}
