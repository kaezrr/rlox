use std::{
    cell::RefCell,
    collections::HashMap,
    io::stdin,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Scope,
    interpreter::{ExecResult, ExecSignal, Interpreter},
    stmt::Stmt,
    token::{Literal, Token},
};

enum Kind {
    NativeFunction(NativeFunction),
    LoxFunction(LoxFunction),
}

pub struct Callable {
    pub arity: usize,
    pub name: String,
    kind: Kind,
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> ExecResult {
        match &self.kind {
            Kind::NativeFunction(native_fn) => native_fn.call(),
            Kind::LoxFunction(user_function) => user_function.call(interpreter, args),
        }
    }

    pub fn lox_function(name: &str, params: Vec<Token>, body: Vec<Stmt>, closure: Rc<RefCell<Scope>>) -> Self {
        Self {
            arity: params.len(),
            name: format!("<fn {}>", name),
            kind: Kind::LoxFunction(LoxFunction { params, body, closure }),
        }
    }
}

pub struct LoxFunction {
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Scope>>,
}

impl LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> ExecResult {
        let mut local_data = HashMap::new();
        for (param, arg) in self.params.iter().zip(args) {
            local_data.insert(param.lexeme.clone(), arg);
        }

        let call_scope = Rc::new(RefCell::new(Scope {
            values: local_data,
            enclosing: Some(self.closure.clone()),
        }));

        interpreter.execute_block(&self.body, call_scope)
    }
}

enum NativeFunction {
    NativeClock(NativeClock),
    ReadNumber(ReadNumber),
    ReadString(ReadString),
}

impl NativeFunction {
    pub fn call(&self) -> ExecResult {
        match self {
            NativeFunction::NativeClock(clock) => clock.call(),
            NativeFunction::ReadNumber(read_num) => read_num.call(),
            NativeFunction::ReadString(read_str) => read_str.call(),
        }
    }
}

pub struct ReadNumber;
pub struct ReadString;
pub struct NativeClock;

impl NativeClock {
    pub fn as_callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 0,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::NativeClock(Self)),
        })
    }

    fn call(&self) -> ExecResult {
        Ok(ExecSignal::Return(Literal::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        )))
    }
}

impl ReadNumber {
    pub fn as_callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 0,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::ReadNumber(Self)),
        })
    }

    fn call(&self) -> ExecResult {
        Ok(ExecSignal::Return(Literal::Number({
            let mut line = String::new();
            stdin().read_line(&mut line).unwrap();
            line.trim().parse().unwrap()
        })))
    }
}

impl ReadString {
    pub fn as_callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 0,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::ReadString(Self)),
        })
    }

    fn call(&self) -> ExecResult {
        Ok(ExecSignal::Return(Literal::String({
            let mut line = String::new();
            stdin().read_line(&mut line).unwrap();
            line.trim_end_matches("\n").to_string()
        })))
    }
}
