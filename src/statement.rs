
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expression;
use crate::token::Token;
use crate::expression::Value;
use crate::environment::Environment;
use crate::callable::LoxCallable;
use crate::util::UnwindType;
use crate::lox_class::LoxClass;
use crate::expression::ClassType;
use crate::error_reporter::ERROR_REPORTER;

#[derive(PartialEq)]
pub enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Clone)]
pub enum Statement {
    Expression {
        expression: expression::Expression,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Statement>,
    },
    Print {
        expression: expression::Expression,
    },
    Return {
        keyword: Token,
        value: Option<expression::Expression>,
    },
    Var {
        name: Token,
        initializer: Option<expression::Expression>,
    },
    Block {
        statements: Vec<Statement>,    
    },
    If {
        condition: expression::Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: expression::Expression,
        body: Box<Statement>,
    },
    Class {
        name: Token,
        methods: Vec<Statement>,
    },
}

impl Statement {
    pub fn interpret(&self, environment: Rc<RefCell<Environment>>) -> Result<(), UnwindType> {
        match self {
            Statement::Expression{expression} => {expression.interpret(environment)?;},
            Statement::Print{expression} => println!("{}", expression.interpret(environment)?),
            Statement::Var{name, initializer} => {
                let value = if let Some(init) = initializer { init.interpret(environment.clone())? } else { Value::Nil };
                environment.borrow_mut().define(name.lexeme().to_string(), value);
            },
            Statement::Block{statements} => {
                let scoped_environment = Rc::new(RefCell::new(Environment::with_enclosing_scope(environment.clone())));
                for statement in statements {
                    statement.interpret(scoped_environment.clone())?;
                }
            },
            Statement::If{condition, then_branch, else_branch} => {
                if condition.interpret(environment.clone())?.is_truthy() {
                    then_branch.interpret(environment)?;
                } else {
                    if let Some(branch) = else_branch {
                        branch.interpret(environment)?;
                    }
                }
            },
            Statement::While{condition, body} => {
                while condition.interpret(environment.clone())?.is_truthy() {
                    body.interpret(environment.clone())?;
                }
            },
            Statement::Function{name, params, body} => {
                environment.borrow_mut().define(name.lexeme().to_string(), Value::Callable(Rc::new(LoxCallable::new(name.clone(), params.clone(), body.clone(), environment.clone(), false))));
            },
            Statement::Return{keyword: _, value} => {
                if let Some(expr) = value {
                    return Err(UnwindType::Return(expr.interpret(environment.clone())?))
                } else {
                    return Err(UnwindType::Return(Value::Nil))
                }
            },
            Statement::Class{name, methods} => {
                environment.borrow_mut().define(name.lexeme().to_string(), Value::Nil);
                let mut final_methods = HashMap::new();
                for method in methods {
                    match method {
                        Statement::Function{name: method_name, params, body} => { final_methods.insert(method_name.lexeme().to_string(), Rc::new(LoxCallable::new(method_name.clone(), params.clone(), body.clone(), environment.clone(), method_name.lexeme() == "init"))); },
                        _ => panic!("An invalid method snuck in!"),
                    }
                }
                environment.borrow_mut().define(name.lexeme().to_string(), Value::Callable(Rc::new(LoxClass::new(name.lexeme().to_string(), final_methods))));
            }
        }
        Ok(())
    }

    pub fn resolve(&mut self, scopes: &mut Vec<HashMap<String, bool>>, function_type: &FunctionType, class_type: &ClassType) {
        match self {
            Statement::Block{statements} => {
                scopes.push(HashMap::new());
                for statement in statements {
                    statement.resolve(scopes, function_type, class_type);
                }
                scopes.pop();
            },
            Statement::Var{name, initializer} => {
                if let Some(last) = scopes.last_mut() {
                    if last.contains_key(name.lexeme()) {
                        ERROR_REPORTER.lock().unwrap().error_on_token(name, "A variable with this name already exists in this scope.");
                    }
                    last.insert(name.lexeme().to_string(), false);
                }
                if let Some(init) = initializer {
                    init.resolve(scopes, class_type);
                }
                if let Some(last) = scopes.last_mut() {
                    last.insert(name.lexeme().to_string(), true);
                }
            },
            Statement::Function{name, params, body} => {
                if let Some(last) = scopes.last_mut() {
                    if last.contains_key(name.lexeme()) {
                        ERROR_REPORTER.lock().unwrap().error_on_token(name, "A variable with this name already exists in this scope.");
                    }
                    last.insert(name.lexeme().to_string(), true);
                }
                let new_function_type = FunctionType::Function;
                scopes.push(HashMap::new());
                for param in params {
                    if let Some(last) = scopes.last_mut() {
                        if last.contains_key(param.lexeme()) {
                            ERROR_REPORTER.lock().unwrap().error_on_token(param, "A variable with this name already exists in this scope.");
                        }
                        last.insert(param.lexeme().to_string(), true);
                    }
                }
                for statement in body {
                    statement.resolve(scopes, &new_function_type, class_type);
                }
                scopes.pop();
            },
            Statement::Expression{expression} => expression.resolve(scopes, class_type),
            Statement::If{condition, then_branch, else_branch} => {
                condition.resolve(scopes, class_type);
                then_branch.resolve(scopes, function_type, class_type);
                if let Some(branch) = else_branch {
                    branch.resolve(scopes, function_type, class_type);
                }
            },
            Statement::Print{expression} => expression.resolve(scopes, class_type),
            Statement::Return{keyword, value} => {
                if *function_type == FunctionType::None {
                    ERROR_REPORTER.lock().unwrap().error_on_token(keyword, "Can't return from top-level code.");
                }
                if *function_type == FunctionType::Initializer && value.is_some() {
                    ERROR_REPORTER.lock().unwrap().error_on_token(keyword, "Can't return a value from an initializer.");
                }
                if let Some(expr) = value {
                    expr.resolve(scopes, class_type)
                }
            },
            Statement::While{condition, body} => {
                condition.resolve(scopes, class_type);
                body.resolve(scopes, function_type, class_type);
            },
            Statement::Class{name, methods} => {
                if let Some(last) = scopes.last_mut() {
                    if last.contains_key(name.lexeme()) {
                        ERROR_REPORTER.lock().unwrap().error_on_token(name, "A variable with this name already exists in this scope.");
                    }
                    last.insert(name.lexeme().to_string(), false);
                }
                let new_class_type = ClassType::Class;
                scopes.push(HashMap::new());
                scopes.last_mut().unwrap().insert("this".to_string(), true);
                for method in methods {
                    match method {
                        Statement::Function{name: method_name, params, body} => {
                            let new_function_type = if method_name.lexeme() == "init" {
                                FunctionType::Initializer
                            } else {
                                FunctionType::Method
                            };
                            for param in params {
                                if let Some(last) = scopes.last_mut() {
                                    if last.contains_key(param.lexeme()) {
                                        ERROR_REPORTER.lock().unwrap().error_on_token(param, "A variable with this name already exists in this scope.");
                                    }
                                    last.insert(param.lexeme().to_string(), true);
                                }
                            }
                            for statement in body {
                                statement.resolve(scopes, &new_function_type, &new_class_type);
                            }
                            scopes.pop();
                        }
                        _ => panic!("An invalid method snuck in!"),
                    }
                }
                scopes.pop();
            },
        }
    }
}
