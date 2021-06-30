use std::rc::Rc;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{environment::Environment, token};
use crate::expression::Value;
use crate::token::Token;
use crate::statement::Statement;
use crate::util::UnwindType;
use crate::lox_class::LoxInstance;
use crate::error_reporter::ERROR_REPORTER;

pub trait Callable {
    fn call(self: Rc<Self>, environment: Rc<RefCell<Environment>>, arguments: Vec<Value>) -> Option<Value>;
    fn arity(&self) -> usize;
}

pub struct NativeClock {

}

impl NativeClock {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Callable for NativeClock {
    fn arity(&self) -> usize {
        0
    }

    fn call(self: Rc<Self>, _: Rc<RefCell<Environment>>, _: Vec<Value>) -> Option<Value> {
        if let Ok(time) = SystemTime::now().duration_since(UNIX_EPOCH) {
            Some(Value::Number(time.as_millis() as f64 / 1000.0))
        } else {
            ERROR_REPORTER.lock().unwrap().runtime_error("Unable to determine offset from UNIX epoch: Time is going backwards!");
            None
        }
    }
}

pub struct LoxCallable {
    name: Token,
    params: Vec<Token>,
    body: Vec<Statement>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxCallable {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Statement>, closure: Rc<RefCell<Environment>>, is_initializer: bool) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> LoxCallable {
        let mut environment = Environment::with_enclosing_scope(self.closure.clone());
        environment.define("this".to_string(), Value::Instance(instance));
        LoxCallable::new(self.name.clone(), self.params.clone(), self.body.clone(), Rc::new(RefCell::new(environment)), self.is_initializer)
    }
}

impl Callable for LoxCallable {
    fn call(self: Rc<Self>, _: Rc<RefCell<Environment>>, arguments: Vec<Value>) -> Option<Value> {
        let scoped_environment = Rc::new(RefCell::new(Environment::with_enclosing_scope(self.closure.clone())));
        for i in 0..self.params.len() {
            scoped_environment.borrow_mut().define(self.params.get(i).unwrap().lexeme().to_string(), arguments.get(i).unwrap().clone());
        }
        for statement in &self.body {
            match statement.interpret(scoped_environment.clone()) {
                Err(UnwindType::Error) => return None,
                Err(UnwindType::Return(value)) => {
                    if self.is_initializer {
                        // This is a bit of a hack. Let's hope resolution dosen't magically fail, or the error
                        // message will be strange!
                        return self.closure.borrow().get_at(Some(0), &Token::new(token::Type::This, "this".to_string(), 0));
                    }
                    return Some(value)
                },
                Ok(()) => {},
            }
        };
        if self.is_initializer {
            // This is a bit of a hack. Let's hope resolution dosen't magically fail, or the error
            // message will be strange!
            self.closure.borrow().get_at(Some(0), &Token::new(token::Type::This, "this".to_string(), 0))
        } else {
            Some(Value::Nil)
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}
