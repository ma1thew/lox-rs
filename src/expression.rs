use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::token;
use crate::token::Token;
use crate::environment::Environment;
use crate::callable;
use crate::util::UnwindType;
use crate::lox_class::LoxInstance;
use crate::error_reporter::ERROR_REPORTER;

#[derive(Clone)]
pub enum Value {
    String(String),
    Number(f64),
    True,
    False,
    Nil,
    Callable(Rc<dyn callable::Callable>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Value::String(ref l), &Value::String(ref r)) => l == r,
            (&Value::Number(l), &Value::Number(r)) => l == r,
            (&Value::True, &Value::True) => true,
            (&Value::False, &Value::False) => true,
            (&Value::Nil, &Value::Nil) => true,
            (&Value::Callable(ref l), &Value::Callable(ref r)) => Rc::ptr_eq(&l, &r),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::False => write!(f, "false"),
            Value::True => write!(f, "true"),
            Value::Nil => write!(f, "nil"),
            Value::Callable(func) => write!(f, "callable {:?}({} arguments)", Rc::as_ptr(func), func.arity()),
            Value::Instance(obj) => write!(f, "{}", obj.borrow()),
        }
    }
}

impl Value {
    pub fn from_bool(value: bool) -> Self {
        match value {
            true => Value::True,
            false => Value::False,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::False => false,
            _ => true,
        }
    }

    pub fn not(&self) -> Value {
        match self.is_truthy() {
            true => Value::False,
            false => Value::True,
        }
    }

    pub fn as_number(&self, operator: Option<&Token>) -> Result<f64, UnwindType> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => {
                if let Some(token) = operator {
                    ERROR_REPORTER.lock().unwrap().runtime_error_on_token(token, "Operand must be a number.");
                }
                Err(UnwindType::Error)
            },
        }
    }
}

#[derive(PartialEq)]
pub enum ClassType {
    None,
    Class,
}

#[derive(Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren: Token,
        arguments: Vec<Expression>,
    },
    Grouping {
        expression: Box<Expression>,
    },
    Literal {
        value: Value,
    },
    Unary {
        operator: Token,
        right: Box<Expression>,
    },
    Variable {
        name: Token,
        depth: Option<usize>,
    },
    Assignment {
        name: Token,
        value: Box<Expression>,
        depth: Option<usize>,
    },
    Logical {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Get {
        object: Box<Expression>,
        name: Token,
    },
    Set {
        object: Box<Expression>,
        name: Token,
        value: Box<Expression>,
    },
    This {
        keyword: Token,
        depth: Option<usize>,
    },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Binary{left, operator, right}  => write!(f, "({} {} {})", operator.lexeme(), left, right),
            Expression::Call{callee, paren: _, arguments} => write!(f, "(call {} {:?})", callee, arguments),
            Expression::Grouping{expression}  => write!(f, "(group {})", expression),
            Expression::Literal{value} => write!(f, "{}", value),
            Expression::Unary{operator, right} => write!(f, "({} {})", operator.lexeme(), right),
            Expression::Variable{name, depth: _} => write!(f, "(variable {})", name.lexeme()),
            Expression::Assignment{name, value, depth: _} => write!(f, "(assign {} {})", name.lexeme(), value),
            Expression::Logical{left, operator, right}  => write!(f, "({} {} {})", operator.lexeme(), left, right),
            Expression::Get{object, name} => write!(f, "(property {} {})", object, name),
            Expression::Set{object, name, value} => write!(f, "(property set {} {} {})", object, name, value),
            Expression::This{keyword, depth: _} => write!(f, "{}", keyword.lexeme()),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Expression {
    pub fn interpret(&self, environment: Rc<RefCell<Environment>>) -> Result<Value, UnwindType> {
        match self {
            Expression::Literal{value} => Ok(value.clone()),
            Expression::Grouping{expression} => expression.interpret(environment),
            Expression::Unary{operator, right} => {
                let right = right.interpret(environment)?;
                match operator.token_type() {
                    token::Type::Minus => Ok(Value::Number(right.as_number(Some(operator))? * -1.0)),
                    token::Type::Bang => Ok(right.not()),
                    _ => panic!("An invalid unary operator snuck in!")
                }
            },
            Expression::Binary{left, operator, right} => {
                let left = left.interpret(environment.clone())?;
                let right = right.interpret(environment.clone())?;

                match operator.token_type() {
                    token::Type::Greater => Ok(Value::from_bool(left.as_number(Some(operator))? > right.as_number(Some(operator))?)),
                    token::Type::GreaterEqual => Ok(Value::from_bool(left.as_number(Some(operator))? >= right.as_number(Some(operator))?)),
                    token::Type::Less => Ok(Value::from_bool(left.as_number(Some(operator))? < right.as_number(Some(operator))?)),
                    token::Type::LessEqual => Ok(Value::from_bool(left.as_number(Some(operator))? <= right.as_number(Some(operator))?)),
                    token::Type::BangEqual => Ok(Value::from_bool(left != right)),
                    token::Type::EqualEqual => Ok(Value::from_bool(left == right)),
                    token::Type::Minus => Ok(Value::Number(left.as_number(Some(operator))? - right.as_number(Some(operator))?)),
                    token::Type::Slash => Ok(Value::Number(left.as_number(Some(operator))? / right.as_number(Some(operator))?)),
                    token::Type::Star => Ok(Value::Number(left.as_number(Some(operator))? * right.as_number(Some(operator))?)),
                    token::Type::Plus => {
                        match (left, right) {
                            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                            (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                            _ => {
                                ERROR_REPORTER.lock().unwrap().runtime_error_on_token(operator, "Operands must be either two numbers or two strings.");
                                Err(UnwindType::Error)
                            },
                        }
                    },
                    _ => panic!("An invalid binary operator snuck in!")
                }
            },
            Expression::Variable{name, depth} => environment.borrow().get_at(*depth, name).ok_or(UnwindType::Error),
            Expression::Assignment{name, value, depth} => {
                let value = value.interpret(environment.clone())?;
                environment.borrow_mut().assign_at(*depth, name.clone(), value.clone()).ok_or(UnwindType::Error)?;
                Ok(value)
            },
            Expression::Logical{left, operator, right} => {
                let left = left.interpret(environment.clone())?;
                match operator.token_type() {
                    token::Type::Or => if left.is_truthy() { Ok(left) } else { right.interpret(environment) },
                    token::Type::And => if !left.is_truthy() { Ok(left) } else { right.interpret(environment) },
                    _ => panic!("An invalid logical operator snuck in!"),
                }
            },
            Expression::Call{callee, paren, arguments} => {
                let callee = callee.interpret(environment.clone())?;
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(argument.interpret(environment.clone())?);
                }
                match callee {
                    Value::Callable(func) => {
                        if args.len() != func.arity() {
                            ERROR_REPORTER.lock().unwrap().runtime_error_on_token(paren, &format!("Expected {} arguments but got {}.", func.arity(), args.len()));
                            Err(UnwindType::Error)
                        } else {
                            func.call(environment.clone(), args).ok_or(UnwindType::Error)
                        }
                    }
                    _ => {
                        ERROR_REPORTER.lock().unwrap().runtime_error_on_token(paren, "Can only call functions and classes.");
                        Err(UnwindType::Error)
                    }
                }
            },
            Expression::Get{object, name} => {
                let object = object.interpret(environment.clone())?;
                match object {
                    Value::Instance(inst) => {
                        let inst_ref = inst.clone();
                        inst.borrow().get(name, inst_ref).ok_or(UnwindType::Error)
                    },
                    _ => {
                        ERROR_REPORTER.lock().unwrap().error_on_token(name, "Only instances have properties.");
                        Err(UnwindType::Error)
                    },
                }
            },
            Expression::Set{object, name, value} => {
                let object = object.interpret(environment.clone())?;
                match object {
                    Value::Instance(inst) => {
                        let value = value.interpret(environment)?;
                        inst.borrow_mut().set(name, value.clone());
                        Ok(value)
                    },
                    _ => {
                        ERROR_REPORTER.lock().unwrap().error_on_token(name, "Only instances have properties.");
                        Err(UnwindType::Error)
                    },
                }
            },
            Expression::This{keyword, depth} => environment.borrow().get_at(*depth, keyword).ok_or(UnwindType::Error),
        }
    }

    pub fn resolve(&mut self, scopes: &mut Vec<HashMap<String, bool>>, class_type: &ClassType) {
        match self {
            Expression::Variable{name, depth} => {
                if let Some(last) = scopes.last() {
                    if let Some(is_defined) = last.get(name.lexeme()) {
                        if !is_defined {
                            ERROR_REPORTER.lock().unwrap().error_on_token(name, "Can't read local variable in it's own initializer.");
                        }
                    }
                }
                for i in (0..scopes.len()).rev() {
                    if scopes.get(i).unwrap().contains_key(name.lexeme()) {
                        *depth = Some(scopes.len() - 1 - i);
                        break;
                    }
                }
            },
            Expression::Assignment{name, value, depth} => {
                value.resolve(scopes, class_type);
                for i in (0..scopes.len()).rev() {
                    if scopes.get(i).unwrap().contains_key(name.lexeme()) {
                        *depth = Some(scopes.len() - 1 - i);
                        break;
                    }
                }
            },
            Expression::Binary{left, operator: _, right} => {
                left.resolve(scopes, class_type);
                right.resolve(scopes, class_type);
            },
            Expression::Call{callee, paren: _, arguments} => {
                callee.resolve(scopes, class_type);
                for argument in arguments {
                    argument.resolve(scopes, class_type);
                }
            },
            Expression::Grouping{expression} => expression.resolve(scopes, class_type),
            Expression::Literal{value: _} => {},
            Expression::Logical{left, operator: _, right} => {
                left.resolve(scopes, class_type);
                right.resolve(scopes, class_type);
            },
            Expression::Unary{operator: _, right} => right.resolve(scopes, class_type),
            Expression::Get{object, name: _} => object.resolve(scopes, class_type),
            Expression::Set{object, name: _, value} => {
                value.resolve(scopes, class_type);
                object.resolve(scopes, class_type);
            },
            Expression::This{keyword, depth} => {
                if *class_type == ClassType::None {
                    ERROR_REPORTER.lock().unwrap().error_on_token(keyword, "Can't use 'this' outside of a class.")
                } else {
                    for i in (0..scopes.len()).rev() {
                        if scopes.get(i).unwrap().contains_key(keyword.lexeme()) {
                            *depth = Some(scopes.len() - 1 - i);
                            break;
                        }
                    }
                }
            },
        }
    }
}
