use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    callable::{Callable, Kind, LoxFunction},
    interpreter::{EvalResult, ExecResult, ExecSignal, Interpreter, RuntimeError},
    token::{Literal, Token},
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn callable(self, arity: usize) -> Rc<Callable> {
        Rc::new(Callable {
            arity,
            name: format!("<class {}>", self.name),
            kind: Kind::Class(self),
        })
    }

    pub fn new(name: String, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> ExecResult {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            let init = initializer.bind(instance.clone()).callable_method(&self.name, "init");
            init.call(interpreter, args)?;
        }

        Ok(ExecSignal::Return(Literal::Instance(instance)))
    }

    fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
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

    pub fn get(&self, name: &Token, instance_rc: Rc<RefCell<Self>>) -> EvalResult {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(method) = self.class.find_method(&name.lexeme) {
            let method = method.bind(instance_rc).callable_method(&self.class.name, &name.lexeme);
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
