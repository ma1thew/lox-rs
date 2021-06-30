use std::rc::Rc;
use std::cell::RefCell;

use crate::expression::Value;
use crate::statement::Statement;
use crate::environment::Environment;
use crate::callable::NativeClock;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().define("clock".to_owned(), Value::Callable(Rc::new(NativeClock::new())));
        Self {
            environment
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            if let Err(_) = statement.interpret(self.environment.clone()) {
                break;
            }
        }
    }
}
