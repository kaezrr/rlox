use crate::{
    environment::Scope,
    expr::{self, Expr},
    stmt::{self, Stmt},
    token::{Literal, Token, TokenType},
};

#[derive(Default)]
pub struct Interpreter {
    scope: Scope,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &[Stmt]) -> ExecResult {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> ExecResult {
        stmt.accept(self)
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> ExecResult {
        self.scope.push();

        let result = statements.iter().try_for_each(|s| self.execute(s));

        self.scope.pop();

        result
    }

    pub fn evaluate(&mut self, expression: &Expr) -> EvalResult {
        expression.accept(self)
    }
}

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

type EvalResult = Result<Literal, RuntimeError>;
type ExecResult = Result<(), RuntimeError>;

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

            TokenType::BangEqual => Ok(Literal::Boolean(left.is_truthy() != right.is_truthy())),

            TokenType::EqualEqual => Ok(Literal::Boolean(left.is_truthy() == right.is_truthy())),

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

    fn visit_variable(&mut self, name: &Token) -> EvalResult {
        self.scope.get(name)
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> EvalResult {
        let value = self.evaluate(value)?;
        self.scope.assign(name, value)
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
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ExecResult {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> ExecResult {
        let value = match initializer {
            Some(v) => self.evaluate(v)?,
            None => Literal::Nil,
        };

        self.scope.define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_block(&mut self, stmts: &[Stmt]) -> ExecResult {
        self.execute_block(stmts)
    }

    fn visit_if_else(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> ExecResult {
        let condition = self.evaluate(condition)?.is_truthy();

        if condition {
            self.execute(then_branch)
        } else if let Some(branch) = else_branch {
            self.execute(branch)
        } else {
            Ok(())
        }
    }
}
