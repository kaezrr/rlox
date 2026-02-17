use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    token::{Literal, Token},
};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, RuntimeError> {
        self.values.get(&name.lexeme).cloned().ok_or(RuntimeError::new(
            name,
            &format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            return Ok(value);
        }

        Err(RuntimeError::new(
            name,
            &format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
