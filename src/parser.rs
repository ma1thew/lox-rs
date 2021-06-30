use std::mem;

use crate::token;
use crate::token::Token;
use crate::expression::Value;
use crate::expression::Expression;
use crate::statement::Statement;
use crate::util::MAXIMUM_PARAMETER_COUNT;
use crate::error_reporter::ERROR_REPORTER;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(statement) = self.declaration() {
               statements.push(statement);
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Statement> {
        let statement = if self.match_types(&[token::Type::Class]) {
            self.class_declaration()
        } else if self.match_types(&[token::Type::Fun]) {
            self.function("function")
        } else if self.match_types(&[token::Type::Var]) {
            self.variable_declaration()
        } else {
            self.statement()
        };
        if let None = statement {
            self.synchronize()
        }
        statement
    }

    fn class_declaration(&mut self) -> Option<Statement> {
        let name = self.consume(&token::Type::Identifier, "Expected class name.")?.clone();
        self.consume(&token::Type::LeftBrace, "Expected '{' before class body.")?;
        let mut methods = Vec::new();
        while !self.check(&token::Type::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(&token::Type::RightBrace, "Expected '}' after class body.")?;
        Some(Statement::Class {
            name,
            methods,
        })
    }

    fn function(&mut self, kind: &str) -> Option<Statement> {
        let name = self.consume(&token::Type::Identifier, &format!("Expected {} name.", kind))?.clone();
        self.consume(&token::Type::LeftParen, &format!("Expected '(' after {} name.", kind))?;
        let mut parameters = Vec::new();
        if !self.check(&token::Type::RightParen) {
            loop {
                if parameters.len() >= MAXIMUM_PARAMETER_COUNT {
                    // No need to return None and unwind; the parser is not confused.
                    ERROR_REPORTER.lock().unwrap().runtime_error_on_token(self.peek(), "Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(&token::Type::Identifier, "Expected parameter name.")?.clone());
                if !self.match_types(&[token::Type::Comma]) { break; }
            }
        }
        self.consume(&token::Type::RightParen, "Expected ')' after parameters.")?;
        self.consume(&token::Type::LeftBrace, &format!("Expected '{{' before {} body.", kind))?;
        Some(Statement::Function {
            name,
            params: parameters,
            body: self.block_statement()?,
        })
    }

    fn variable_declaration(&mut self) -> Option<Statement> {
        let name = self.consume(&token::Type::Identifier, "Expected variable name!")?.clone();
        let initializer = if self.match_types(&[token::Type::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&token::Type::Semicolon, "Expected ';' after variable declaration")?;
        Some(Statement::Var{name, initializer})
    }

    fn statement(&mut self) -> Option<Statement> {
        if self.match_types(&[token::Type::For]) {
            self.for_statement()
        } else if self.match_types(&[token::Type::If]) {
            self.if_statement()
        } else if self.match_types(&[token::Type::Print]) {
            self.print_statement()
        } else if self.match_types(&[token::Type::Return]) {
            self.return_statement()
        } else if self.match_types(&[token::Type::While]) {
            self.while_statement()
        } else if self.match_types(&[token::Type::LeftBrace]) {
            Some(Statement::Block{statements: self.block_statement()?})
        } else {
            self.expression_statement()
        }
    }

    // There is no such thing as a for statement! This desugars for-loop syntax into a while loop
    // inside a block!
    fn for_statement(&mut self) -> Option<Statement> {
        self.consume(&token::Type::LeftParen, "Expected '(' after 'for'.")?;
        let initializer = if self.match_types(&[token::Type::Semicolon]) {
            None
        } else if self.match_types(&[token::Type::Var]) {
            Some(self.variable_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if !self.check(&token::Type::Semicolon) {
            self.expression()?
        } else {
            Expression::Literal{value: Value::True}
        };
        self.consume(&token::Type::Semicolon, "Expected ';' after loop condition.")?;
        let increment = if !self.check(&token::Type::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&token::Type::RightParen, "Expected ')' after clauses.")?;
        let mut body = self.statement()?;
        if let Some(incr) = increment {
            body = Statement::Block {
                statements: vec![body, Statement::Expression{expression: incr}],
            };
        }
        body = Statement::While{condition, body: Box::new(body)};
        if let Some(init) = initializer {
            body = Statement::Block{
                statements: vec![init, body]
            };
        }
        Some(body)
    }

    fn if_statement(&mut self) -> Option<Statement> {
        self.consume(&token::Type::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&token::Type::RightParen, "Expected ')' after if condition.")?;
        Some(Statement::If {
            condition,
            then_branch: Box::new(self.statement()?),
            else_branch: if self.match_types(&[token::Type::Else]) { Some(Box::new(self.statement()?)) } else { None },
        })
    }

    fn print_statement(&mut self) -> Option<Statement> {
        let value = self.expression()?;
        self.consume(&token::Type::Semicolon, "Expected ';' after value.")?;
        Some(Statement::Print{expression: value})
    }

    fn return_statement(&mut self) -> Option<Statement> {
        let keyword = self.previous().clone();
        let value = if !self.check(&token::Type::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&token::Type::Semicolon, "Expected ';' after return value.")?;
        Some(Statement::Return {
            keyword,
            value,
        })
    }

    fn while_statement(&mut self) -> Option<Statement> {
        self.consume(&token::Type::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&token::Type::RightParen, "Expected ')' after condition.")?;
        Some(Statement::While{condition, body: Box::new(self.statement()?)})
    }

    fn block_statement(&mut self) -> Option<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.check(&token::Type::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&token::Type::RightBrace, "Expected '}' after block.")?;
        Some(statements)
    }

    fn expression_statement(&mut self) -> Option<Statement> {
        let expression = self.expression()?;
        self.consume(&token::Type::Semicolon, "Expected ';' after expression.")?;
        Some(Statement::Expression{expression})
    }

    fn expression(&mut self) -> Option<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expression> {
        let expr = self.or()?;
        if self.match_types(&[token::Type::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            match expr {
                Expression::Variable{name, depth} => Some(Expression::Assignment{name, value: Box::new(value), depth}),
                Expression::Get{object, name} => Some(Expression::Set{object, name, value: Box::new(value)}),
                _ => {
                    // Note that we report an error, but don't propgate it further; this is because
                    // we *don't* need to synchronize.
                    ERROR_REPORTER.lock().unwrap().error_on_token(&equals, "Invalid assignment target.");
                    Some(expr)
                },
            }
        } else {
            Some(expr)
        }
    }

    fn or(&mut self) -> Option<Expression> {
        let mut expr = self.and()?;
        while self.match_types(&[token::Type::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expression::Logical{left: Box::new(expr), operator, right: Box::new(right)}
        }
        Some(expr)
    }

    fn and(&mut self) -> Option<Expression> {
        let mut expr = self.equality()?;
        while self.match_types(&[token::Type::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expression::Logical{left: Box::new(expr), operator, right: Box::new(right)}
        }
        Some(expr)
    }

    fn equality(&mut self) -> Option<Expression> {
        let mut expr = self.comparison()?;
        while self.match_types(&[token::Type::BangEqual, token::Type::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expression::Binary{left: Box::new(expr), operator, right: Box::new(right)};
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expression> {
        let mut expr = self.term()?;
        while self.match_types(&[token::Type::Greater, token::Type::GreaterEqual, token::Type::Less, token::Type::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary{left: Box::new(expr), operator: operator.clone(), right: Box::new(right)};
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expression> {
        let mut expr = self.factor()?;
        while self.match_types(&[token::Type::Minus, token::Type::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary{left: Box::new(expr), operator: operator.clone(), right: Box::new(right)};
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expression> {
        let mut expr = self.unary()?;
        while self.match_types(&[token::Type::Slash, token::Type::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary{left: Box::new(expr), operator: operator.clone(), right: Box::new(right)};
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expression> {
        if self.match_types(&[token::Type::Bang, token::Type::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Some(Expression::Unary{operator: operator.clone(), right: Box::new(right)})
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Option<Expression> {
        let mut expr = self.primary()?;
        loop {
            if self.match_types(&[token::Type::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_types(&[token::Type::Dot]) {
                expr = Expression::Get{object: Box::new(expr), name: self.consume(&token::Type::Identifier, "Expected property name after '.'.")?.clone()}
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Option<Expression> {
        let mut arguments = Vec::new();
        if !self.check(&token::Type::RightParen) {
            loop {
                if arguments.len() >= MAXIMUM_PARAMETER_COUNT {
                    // No need to return None and unwind; the parser isn't confused.
                    ERROR_REPORTER.lock().unwrap().error_on_token(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.match_types(&[token::Type::Comma]) { break; }
            }
        }
        Some(Expression::Call {
            callee: Box::new(callee),
            paren: self.consume(&token::Type::RightParen, "Expected ')' after arguments.")?.clone(),
            arguments,
        })
    }

    fn primary(&mut self) -> Option<Expression> {
        // TODO: This is a little wasteful on the allocations.
        if self.match_types(&[token::Type::False, token::Type::True, token::Type::Nil, token::Type::Number(0.0), token::Type::String(String::new())]) {
            Some(Expression::Literal{value: self.previous().token_type().clone().to_value()})
        } else if self.match_types(&[token::Type::This]) {
            Some(Expression::This{keyword: self.previous().clone(), depth: None})
        } else if self.match_types(&[token::Type::Identifier]) {
            Some(Expression::Variable{name: self.previous().clone(), depth: None})
        } else if self.match_types(&[token::Type::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&token::Type::RightParen, "Expected ')' after expression.")?;
            Some(Expression::Grouping{ expression: Box::new(expr) })
        } else {
            ERROR_REPORTER.lock().unwrap().error_on_token(self.peek(), "Expected expression.");
            None
        }
    }

    fn match_types(&mut self, types: &[token::Type]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &token::Type) -> bool {
        if self.is_at_end() {
            false
        } else {
            mem::discriminant(self.peek().token_type()) == mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        mem::discriminant(self.peek().token_type()) == mem::discriminant(&token::Type::EOF)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn consume(&mut self, token_type: &token::Type, message: &str) -> Option<&Token> {
        if self.check(token_type) {
            Some(self.advance())
        } else {
            ERROR_REPORTER.lock().unwrap().error_on_token(self.peek(), message);
            None
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if mem::discriminant(self.previous().token_type()) == mem::discriminant(&token::Type::Semicolon) {
                return
            }

            match self.peek().token_type() {
                token::Type::Class
                | token::Type::Fun
                | token::Type::Var
                | token::Type::For
                | token::Type::If
                | token::Type::While
                | token::Type::Print
                | token::Type::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
