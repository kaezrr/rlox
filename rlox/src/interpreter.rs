use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{Callable, NativeClock, ReadNumber, ReadString},
    environment::Scope,
    expr::{self, Expr, ExprId},
    stmt::{self, Stmt},
    token::{Literal, Token, TokenType},
};

pub struct Interpreter {
    current_scope: Rc<RefCell<Scope>>,
    locals: HashMap<ExprId, usize>,
    globals: Rc<RefCell<Scope>>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut globals = Scope::default();
        globals.define("clock".into(), Literal::Callable(NativeClock::as_callable()));
        globals.define("readString".into(), Literal::Callable(ReadString::as_callable()));
        globals.define("readNumber".into(), Literal::Callable(ReadNumber::as_callable()));

        let global_scope = Rc::new(RefCell::new(globals));

        Self {
            current_scope: global_scope.clone(),
            locals: Default::default(),
            globals: global_scope.clone(),
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Stmt]) -> ExecResult {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(ExecSignal::None)
    }

    fn execute(&mut self, stmt: &Stmt) -> ExecResult {
        stmt.accept(self)
    }

    pub fn execute_block(&mut self, statements: &[Stmt], env: Rc<RefCell<Scope>>) -> ExecResult {
        let previous = self.current_scope.clone();
        self.current_scope = env;

        let result = (|| {
            for stmt in statements {
                let res = self.execute(stmt)?;
                if let ExecSignal::None = res {
                    continue;
                }

                return Ok(res);
            }
            Ok(ExecSignal::None)
        })();

        self.current_scope = previous;

        result
    }

    pub fn evaluate(&mut self, expression: &Expr) -> EvalResult {
        expression.accept(self)
    }

    fn look_up_variable(&self, name: &Token, expr: &Expr) -> EvalResult {
        let Some(&distance) = self.locals.get(&expr.id) else {
            return self.globals.borrow().get(name);
        };

        Ok(self.current_scope.borrow().get_at(distance, name))
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.id, depth);
    }
}

impl expr::Visitor<EvalResult> for Interpreter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> EvalResult {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Number(left - right))
            }

            TokenType::Slash => {
                let (left, right) = check_number_operands(operator, &left, &right)?;

                if !right.is_normal() {
                    return Err(RuntimeError::new(operator, "Division by zero."));
                }

                Ok(Literal::Number(left / right))
            }

            TokenType::Star => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Number(left * right))
            }

            TokenType::Plus => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Number(left + right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::String(left + &right)),
                (Literal::String(left), Literal::Number(right)) => Ok(Literal::String(left + &right.to_string())),
                (Literal::Number(left), Literal::String(right)) => Ok(Literal::String(left.to_string() + &right)),
                _ => Err(RuntimeError::new(operator, "Operands must be numbers or strings.")),
            },

            TokenType::Greater => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Boolean(left > right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::Boolean(left > right)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::GreaterEqual => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Boolean(left >= right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::Boolean(left >= right)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::Less => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Boolean(left < right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::Boolean(left < right)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::LessEqual => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Boolean(left <= right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::Boolean(left <= right)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::EqualEqual => Ok(Literal::Boolean(is_equal(&left, &right))),

            TokenType::BangEqual => Ok(Literal::Boolean(!is_equal(&left, &right))),

            _ => unreachable!(),
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) -> EvalResult {
        self.evaluate(expression)
    }

    fn visit_literal(&mut self, literal: &Literal) -> EvalResult {
        Ok(literal.clone())
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> EvalResult {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => {
                let n = check_number_operand(operator, &right)?;
                Ok(Literal::Number(-n))
            }
            TokenType::Bang => Ok(Literal::Boolean(!right.is_truthy())),
            _ => unreachable!("unary eval"),
        }
    }

    fn visit_comma(&mut self, left: &Expr, right: &Expr) -> EvalResult {
        let _ = self.evaluate(left)?;
        self.evaluate(right)
    }

    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> EvalResult {
        let condition = self.evaluate(cond)?.is_truthy();

        if condition {
            self.evaluate(left)
        } else {
            self.evaluate(right)
        }
    }

    fn visit_variable(&mut self, name: &Token, expr: &Expr) -> EvalResult {
        self.look_up_variable(name, expr)
    }

    fn visit_assign(&mut self, name: &Token, _expr: &Expr, value: &Expr) -> EvalResult {
        let evaled = self.evaluate(value)?;

        let Some(&distance) = self.locals.get(&value.id) else {
            return self.globals.borrow_mut().assign(name, evaled);
        };

        Ok(self.current_scope.borrow_mut().assign_at(distance, name, evaled))
    }

    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> EvalResult {
        let left = self.evaluate(left)?;

        if operator.token_type == TokenType::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else if !left.is_truthy() {
            return Ok(left);
        }

        self.evaluate(right)
    }

    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> EvalResult {
        let callee = self.evaluate(callee)?;
        let mut args = Vec::with_capacity(arguments.len());

        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }

        let Literal::Callable(function) = callee else {
            return Err(RuntimeError::new(paren, "Can only call function and classes."));
        };

        if args.len() != function.arity {
            return Err(RuntimeError::new(
                paren,
                &format!("Expected {} arguments but got {}.", function.arity, args.len()),
            ));
        }

        function.call(self, args).map(|x| match x {
            ExecSignal::None => Literal::Nil,
            ExecSignal::Return(literal) => literal,
            _ => unreachable!(),
        })
    }

    fn visit_lambda(&mut self, name: Option<&Token>, params: &[Token], body: &[Stmt]) -> EvalResult {
        let params = params.to_vec();
        let body = body.to_vec();

        let closure = self.current_scope.clone();

        Ok(Literal::Callable(Rc::new(Callable::lox_function(
            name.map_or("anonymous", |x| &x.lexeme),
            params,
            body,
            closure,
        ))))
    }
}

fn is_equal(left: &Literal, right: &Literal) -> bool {
    match (left, right) {
        (Literal::Number(a), Literal::Number(b)) => a == b,
        (Literal::String(a), Literal::String(b)) => a == b,
        (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
        (Literal::Nil, Literal::Nil) => true,
        (Literal::Callable(a), Literal::Callable(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

fn check_number_operand(operator: &Token, operand: &Literal) -> Result<f64, RuntimeError> {
    if let Literal::Number(n) = operand {
        return Ok(*n);
    }

    Err(RuntimeError::new(operator, "Operand must be a number."))
}

fn check_number_operands(operator: &Token, left: &Literal, right: &Literal) -> Result<(f64, f64), RuntimeError> {
    if let Literal::Number(a) = left
        && let Literal::Number(b) = right
    {
        return Ok((*a, *b));
    }

    Err(RuntimeError::new(operator, "Operands must be numbers."))
}

impl stmt::Visitor<ExecResult> for Interpreter {
    fn visit_print_stmt(&mut self, expr: &Expr) -> ExecResult {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(ExecSignal::None)
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ExecResult {
        self.evaluate(expr)?;
        Ok(ExecSignal::None)
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> ExecResult {
        let value = match initializer {
            Some(v) => self.evaluate(v)?,
            None => Literal::Nil,
        };

        self.current_scope.borrow_mut().define(name.lexeme.clone(), value);
        Ok(ExecSignal::None)
    }

    fn visit_block(&mut self, stmts: &[Stmt]) -> ExecResult {
        let scope = Rc::new(RefCell::new(Scope {
            values: HashMap::new(),
            enclosing: Some(self.current_scope.clone()),
        }));

        self.execute_block(stmts, scope)
    }

    fn visit_if_else(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> ExecResult {
        let condition = self.evaluate(condition)?.is_truthy();

        if condition {
            self.execute(then_branch)
        } else if let Some(branch) = else_branch {
            self.execute(branch)
        } else {
            Ok(ExecSignal::None)
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> ExecResult {
        while self.evaluate(condition)?.is_truthy() {
            if let ExecSignal::Break = self.execute(body)? {
                break;
            }
        }

        Ok(ExecSignal::None)
    }

    fn visit_break(&mut self) -> ExecResult {
        Ok(ExecSignal::Break)
    }

    fn visit_return(&mut self, _keyword: &Token, value: Option<&Expr>) -> ExecResult {
        let value = match value {
            Some(v) => self.evaluate(v)?,
            None => Literal::Nil,
        };

        Ok(ExecSignal::Return(value))
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, err_msg: &str) -> Self {
        Self {
            token: token.clone(),
            message: err_msg.to_string(),
        }
    }
}

pub type EvalResult = Result<Literal, RuntimeError>;
pub type ExecResult = Result<ExecSignal, RuntimeError>;

pub enum ExecSignal {
    None,
    Return(Literal),
    Break,
}
