use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    callable::{Callable, Kind, LoxFunction},
    interpreter::{EvalResult, ExecResult, ExecSignal, Interpreter, RuntimeError},
    token::{Literal, Token},
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    super_class: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
    statics: HashMap<String, Rc<LoxFunction>>,
    getters: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn callable(self, arity: usize) -> Rc<Callable> {
        Rc::new(Callable {
            arity,
            name: format!("<class {}>", self.name),
            kind: Kind::Class(self),
        })
    }

    pub fn new(
        name: String,
        super_class: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
        statics: HashMap<String, Rc<LoxFunction>>,
        getters: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            super_class,
            methods,
            statics,
            getters,
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> ExecResult {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            let init = initializer.bind(instance.clone()).callable_method(&self.name, "init");
            init.call(interpreter, args)?;
        }

        Ok(ExecSignal::Return(Literal::Instance(instance)))
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }

        if let Some(super_class) = &self.super_class {
            return super_class.find_method(name);
        }

        None
    }

    pub fn find_getter(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.getters.get(name).cloned()
    }

    pub fn find_static(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.statics.get(name).cloned()
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

    pub fn get(&self, name: &Token, interpreter: &mut Interpreter, instance_rc: Rc<RefCell<Self>>) -> EvalResult {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(getter) = self.class.find_getter(&name.lexeme) {
            let bound = getter.bind(instance_rc);
            let ExecSignal::Return(value) = bound.call(interpreter, Vec::new())? else {
                return Err(RuntimeError::new(name, "Getter must return a value."));
            };
            return Ok(value);
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
