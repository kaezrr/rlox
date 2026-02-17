use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    token::{Literal, Token},
};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Literal>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn put_enclosing(&mut self, env: Environment) {
        self.enclosing = Some(Box::new(env));
    }

    pub fn take_enclosing(&mut self) -> Option<Box<Environment>> {
        self.enclosing.take()
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = self.enclosing.as_ref() {
            return enclosing.get(name);
        }

        Err(RuntimeError::new(
            name,
            &format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            return Ok(value);
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            return enclosing.assign(name, value);
        }

        Err(RuntimeError::new(
            name,
            &format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
