use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    token::{Literal, Token},
};

type ScopeData = HashMap<String, Literal>;

pub struct Scope {
    environments: Vec<ScopeData>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            environments: vec![ScopeData::default()],
        }
    }
}

impl Scope {
    pub fn push(&mut self) {
        self.environments.push(ScopeData::default());
    }

    pub fn pop(&mut self) {
        debug_assert!(self.environments.len() > 1, "attempted to pop global scope");
        self.environments.pop();
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.environments.last_mut().unwrap().insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, RuntimeError> {
        for env in self.environments.iter().rev() {
            if let Some(value) = env.get(&name.lexeme) {
                return Ok(value.clone());
            }
        }

        Err(undefined_error(name))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal, RuntimeError> {
        for env in self.environments.iter_mut().rev() {
            if let Some(slot) = env.get_mut(&name.lexeme) {
                *slot = value.clone();
                return Ok(value);
            }
        }

        Err(undefined_error(name))
    }
}

fn undefined_error(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}
