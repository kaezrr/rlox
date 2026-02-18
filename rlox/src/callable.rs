use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interpreter::{EvalResult, Interpreter},
    token::Literal,
};

enum Kind {
    NativeFunction(NativeClock),
    UserFunction(UserFunction),
}

pub struct Callable {
    pub arity: usize,
    pub name: String,
    kind: Kind,
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> EvalResult {
        match &self.kind {
            Kind::NativeFunction(native_clock) => Ok(Literal::Number(native_clock.call())),
            Kind::UserFunction(user_function) => todo!(),
        }
    }
}

pub struct NativeClock;

impl NativeClock {
    pub fn as_callable() -> Arc<Callable> {
        Arc::new(Callable {
            arity: 0,
            name: "native_clock".to_string(),
            kind: Kind::NativeFunction(Self),
        })
    }

    fn call(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
}

struct UserFunction;
