use std::{cell::RefCell, fmt, rc::Rc, collections::HashMap};

use crate::{callable::{Callable, LoxCallable}, environment::Environment, error_reporter::ERROR_REPORTER, expression::Value, token::Token};

pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxCallable>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxCallable>>) -> Self {
        Self {
            name,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&Rc<LoxCallable>> {
        self.methods.get(name)
    }
}

impl Callable for LoxClass {
    fn call(self: Rc<Self>, environment: Rc<RefCell<Environment>>, arguments: Vec<Value>) -> Option<Value> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));
        if let Some(init) = self.find_method("init") {
            Rc::new(init.bind(instance.clone())).call(environment, arguments);
        }
        Some(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        if let Some(init) = self.find_method("init") {
            init.arity()
        } else {
            0
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token, this_instance: Rc<RefCell<LoxInstance>>) -> Option<Value> {
        if let Some(value) = self.fields.get(name.lexeme()) {
            Some(value.clone())
        } else if let Some(method) = self.class.find_method(name.lexeme()) {
            Some(Value::Callable(Rc::new(method.clone().bind(this_instance))))
        } else {
            ERROR_REPORTER.lock().unwrap().error_on_token(name, &format!("Undefined property {}.", name.lexeme()));
            None
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme().to_string(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
