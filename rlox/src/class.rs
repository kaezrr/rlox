use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    callable::{Callable, Kind},
    interpreter::{EvalResult, ExecResult, ExecSignal, Interpreter, RuntimeError},
    token::{Literal, Token},
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<Callable>>,
}

impl LoxClass {
    pub fn callable(self) -> Rc<Callable> {
        Rc::new(Callable {
            arity: 0,
            name: format!("<class {}>", self.name),
            kind: Kind::Class(self),
        })
    }

    pub fn new(name: String, methods: HashMap<String, Rc<Callable>>) -> Self {
        Self { name, methods }
    }

    pub fn call(&self, _interpreter: &mut Interpreter, _args: Vec<Literal>) -> ExecResult {
        let instance = LoxInstance::new(self.clone());
        Ok(ExecSignal::Return(Literal::Instance(Rc::new(RefCell::new(instance)))))
    }

    fn find_method(&self, name: &str) -> Option<Rc<Callable>> {
        self.methods.get(name).cloned()
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Literal>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> EvalResult {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(method) = self.class.find_method(&name.lexeme) {
            return Ok(Literal::Callable(method));
        }

        Err(RuntimeError::new(
            name,
            &format!("Undefined property '{}'", name.lexeme),
        ))
    }

    pub fn set(&mut self, name: &Token, value: Literal) -> Literal {
        self.fields.insert(name.lexeme.clone(), value.clone());
        value
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<instance {}>", self.class)
    }
}
