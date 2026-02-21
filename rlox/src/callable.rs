use std::{
    cell::RefCell,
    collections::HashMap,
    io::stdin,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    class::{LoxClass, LoxInstance},
    environment::Scope,
    interpreter::{ExecResult, ExecSignal, Interpreter, RuntimeError},
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

#[derive(Debug)]
pub enum Kind {
    NativeFunction(NativeFunction),
    LoxFunction(LoxFunction),
    Class(LoxClass),
}

#[derive(Debug)]
pub struct Callable {
    pub arity: usize,
    pub name: String,
    pub kind: Kind,
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, paren: &Token, args: Vec<Literal>) -> ExecResult {
        match &self.kind {
            Kind::NativeFunction(native_fn) => native_fn.call(paren, &args),
            Kind::LoxFunction(user_function) => user_function.call(interpreter, args),
            Kind::Class(class) => class.call(interpreter, paren, args),
        }
    }

    pub fn lox_function(
        name: &str,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Scope>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            arity: params.len(),
            name: format!("<fn {}>", name),
            kind: Kind::LoxFunction(LoxFunction {
                params,
                body,
                closure,
                is_initializer,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Scope>>,
    pub is_initializer: bool,
}

impl LoxFunction {
    pub fn callable_method(&self, class_name: &str, name: &str) -> Rc<Callable> {
        Rc::new(Callable {
            arity: self.params.len(),
            name: format!("<fn {}.{}>", class_name, name),
            kind: Kind::LoxFunction(self.clone()),
        })
    }

    pub fn callable_static(&self, class_name: &str, name: &str) -> Rc<Callable> {
        Rc::new(Callable {
            arity: self.params.len(),
            name: format!("<fn static {}.{}>", class_name, name),
            kind: Kind::LoxFunction(self.clone()),
        })
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> ExecResult {
        let mut local_data = HashMap::new();
        for (param, arg) in self.params.iter().zip(args) {
            local_data.insert(param.lexeme.clone(), arg);
        }

        let call_scope = Rc::new(RefCell::new(Scope {
            values: local_data,
            enclosing: Some(self.closure.clone()),
        }));

        let result = interpreter.execute_block(&self.body, call_scope)?;

        if self.is_initializer {
            let this_token = Token::new(TokenType::This, "this".to_string(), None, 1);
            let instance = self.closure.borrow().get_at(0, &this_token);
            return Ok(ExecSignal::Return(instance));
        }

        Ok(result)
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Self {
        let mut scope = Scope::default();
        scope.define("this".to_string(), Literal::Instance(instance));
        scope.enclosing = Some(self.closure.clone());

        Self {
            params: self.params.clone(),
            body: self.body.clone(),
            closure: Rc::new(RefCell::new(scope)),
            is_initializer: self.is_initializer,
        }
    }
}

#[derive(Debug)]
pub enum NativeFunction {
    NativeClock(NativeClock),
    ReadNumber(ReadNumber),
    ReadString(ReadString),
    PushArray(PushArray),
    PopArray(PopArray),
    LenArray(LenArray),
}

impl NativeFunction {
    pub fn call(&self, paren: &Token, args: &[Literal]) -> ExecResult {
        match self {
            NativeFunction::NativeClock(clock) => clock.call(),
            NativeFunction::ReadNumber(read_num) => read_num.call(),
            NativeFunction::ReadString(read_str) => read_str.call(),
            NativeFunction::PushArray(push_array) => push_array.call(paren, args),
            NativeFunction::PopArray(pop_array) => pop_array.call(paren, args),
            NativeFunction::LenArray(len_array) => len_array.call(paren, args),
        }
    }
}

#[derive(Debug)]
pub struct ReadNumber;
#[derive(Debug)]
pub struct ReadString;
#[derive(Debug)]
pub struct NativeClock;

#[derive(Debug)]
pub struct PushArray;
#[derive(Debug)]
pub struct PopArray;
#[derive(Debug)]
pub struct LenArray;

impl NativeClock {
    pub fn callable() -> Rc<Callable> {
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
    pub fn callable() -> Rc<Callable> {
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
    pub fn callable() -> Rc<Callable> {
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

impl PushArray {
    pub fn callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 2,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::PushArray(PushArray)),
        })
    }

    fn call(&self, paren: &Token, args: &[Literal]) -> ExecResult {
        let Literal::List(list) = &args[0] else {
            return Err(RuntimeError::new(paren, "Can only push to lists."));
        };
        list.borrow_mut().push(args[1].clone());
        Ok(ExecSignal::None)
    }
}

impl PopArray {
    pub fn callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 1,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::PopArray(PopArray)),
        })
    }

    fn call(&self, paren: &Token, args: &[Literal]) -> ExecResult {
        let Literal::List(list) = &args[0] else {
            return Err(RuntimeError::new(paren, "Can only pop from lists."));
        };
        Ok(ExecSignal::Return(list.borrow_mut().pop().unwrap_or(Literal::Nil)))
    }
}

impl LenArray {
    pub fn callable() -> Rc<Callable> {
        Rc::new(Callable {
            arity: 1,
            name: "<native fn>".to_string(),
            kind: Kind::NativeFunction(NativeFunction::LenArray(LenArray)),
        })
    }

    fn call(&self, paren: &Token, args: &[Literal]) -> ExecResult {
        let Literal::List(list) = &args[0] else {
            return Err(RuntimeError::new(paren, "Can only get length from lists."));
        };
        Ok(ExecSignal::Return(Literal::Number(list.borrow().len() as f64)))
    }
}
