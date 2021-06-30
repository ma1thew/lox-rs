use std::fmt;

use crate::expression::Value;

#[derive(Debug, Clone)]
pub enum Type {
    // Single character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier, String(String), Number(f64),

    // Keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EOF
}

impl Type {
    pub fn to_value(self) -> Value {
        match self {
            Type::String(s) => Value::String(s),
            Type::Number(n) => Value::Number(n),
            Type::False => Value::False,
            Type::True => Value::True,
            Type::Nil => Value::Nil,
            _ => panic!("Attepted to convert invalid token type to value!")
        }
    }
}

#[derive(Clone)]
pub struct Token {
    token_type: Type,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: Type, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn token_type(&self) -> &Type {
        &self.token_type
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.token_type, self.lexeme)
    }
}
