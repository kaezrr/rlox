use std::{fmt::Display, rc::Rc};

use crate::{
    callable::{Callable, Kind},
    interpreter::{ExecResult, ExecSignal, Interpreter},
    token::Literal,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn callable(self) -> Rc<Callable> {
        Rc::new(Callable {
            arity: 0,
            name: format!("<class {}>", self.name),
            kind: Kind::Class(self),
        })
    }

    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn call(&self, _interpreter: &mut Interpreter, _args: Vec<Literal>) -> ExecResult {
        let instance = LoxInstance::new(self.clone());
        Ok(ExecSignal::Return(Literal::Instance(Rc::new(instance))))
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
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self { class }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
