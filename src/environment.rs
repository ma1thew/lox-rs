use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::token::Token;
use crate::expression::Value;
use crate::error_reporter::ERROR_REPORTER;

pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing_scope(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(value) = self.values.get(name.lexeme()) {
            return Some(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            if let Some(value) = enclosing.borrow().get(name) {
                return Some(value.clone());
            }
        }

        ERROR_REPORTER.lock().unwrap().runtime_error_on_token(name, &format!("Undefined variable '{}'.", name.lexeme()));
        None
    }

    pub fn assign(&mut self, name: Token, value: Value) -> Option<()> {
        if self.values.contains_key(name.lexeme()) {
            self.values.insert(name.lexeme().to_string(), value);
            return Some(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        ERROR_REPORTER.lock().unwrap().runtime_error_on_token(&name, &format!("Undefined variable '{}'.", name.lexeme()));
        None
    }

    pub fn get_at(&self, distance: Option<usize>, name: &Token) -> Option<Value> {
        if let Some(dist) = distance {
            if dist == 0 {
                self.get(name)
            } else {
                self.enclosing.as_ref().unwrap().borrow().get_at(Some(dist - 1), name)
            }
        } else {
            self.get_global(name)
        }
    }

    fn get_global(&self, name: &Token) -> Option<Value> {
        if let Some(encl) = &self.enclosing {
            encl.borrow().get_global(name)
        } else {
            self.get(name)
        }
    }

    pub fn assign_at(&mut self, distance: Option<usize>, name: Token, value: Value) -> Option<()> {
        if let Some(dist) = distance {
            if dist == 0 {
                self.assign(name, value)
            } else {
                self.enclosing.as_ref().unwrap().borrow_mut().assign_at(Some(dist - 1), name, value)
            }
        } else {
            self.assign_global(name, value)
        }
    }


    pub fn assign_global(&mut self, name: Token, value: Value) -> Option<()> {
        if let Some(encl) = &mut self.enclosing {
            encl.borrow_mut().assign_global(name, value)
        } else {
            self.assign(name, value)
        }
    }
}
