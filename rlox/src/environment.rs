use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{EvalResult, RuntimeError},
    token::{Literal, Token},
};

#[derive(Default, Debug)]
pub struct Scope {
    pub values: HashMap<String, Literal>,
    pub enclosing: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> EvalResult {
        if let Some(assigned) = self.values.get(&name.lexeme) {
            return Ok(assigned.clone());
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(undefined_error(name))
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Literal {
        if distance == 0 {
            return self.get(name).unwrap();
        }

        self.ancestor(distance)
            .borrow()
            .values
            .get(&name.lexeme)
            .unwrap()
            .clone()
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> EvalResult {
        if let Some(k) = self.values.get_mut(&name.lexeme) {
            *k = value.clone();
            return Ok(value);
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(undefined_error(name))
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Literal) -> Literal {
        if distance == 0 {
            return self.assign(name, value).unwrap();
        }

        *self
            .ancestor(distance)
            .borrow_mut()
            .values
            .get_mut(&name.lexeme)
            .unwrap() = value.clone();

        value
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Scope>> {
        let mut env = self.enclosing.clone().unwrap();

        for _ in 1..distance {
            let next = env.borrow().enclosing.clone().unwrap();
            env = next;
        }

        env
    }
}

fn undefined_error(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}
