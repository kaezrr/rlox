use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{NativeClock, ReadNumber, ReadString},
    interpreter::RuntimeError,
    token::{Literal, Token},
};

pub struct Scope {
    pub values: HashMap<String, Option<Literal>>,
    pub enclosing: Option<Rc<RefCell<Scope>>>,
}

impl Default for Scope {
    fn default() -> Self {
        let mut global = HashMap::new();
        global.insert("clock".to_string(), Some(Literal::Callable(NativeClock::as_callable())));
        global.insert(
            "read_number".to_string(),
            Some(Literal::Callable(ReadNumber::as_callable())),
        );
        global.insert(
            "read_string".to_string(),
            Some(Literal::Callable(ReadString::as_callable())),
        );

        Self {
            values: global,
            enclosing: None,
        }
    }
}

impl Scope {
    pub fn define(&mut self, name: String, value: Option<Literal>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, RuntimeError> {
        if let Some(assigned) = self.values.get(&name.lexeme) {
            let Some(val) = assigned else {
                return Err(uninitialized_error(name));
            };

            return Ok(val.clone());
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(undefined_error(name))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value.clone()));
            return Ok(value);
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(undefined_error(name))
    }
}

fn undefined_error(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}

fn uninitialized_error(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Variable '{}' is not initialized.", name.lexeme))
}
