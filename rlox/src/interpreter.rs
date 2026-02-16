use crate::{
    expr::{Expr, Visitor},
    token::{Literal, Token, TokenType},
};

#[derive(Default)]
pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&mut self, expression: &Expr) -> EvalResult {
        expression.accept(self)
    }

    fn evaluate(&mut self, expression: &Expr) -> EvalResult {
        expression.accept(self)
    }
}

pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    fn new(token: &Token, err_msg: &str) -> Self {
        Self {
            token: token.clone(),
            message: err_msg.to_string(),
        }
    }
}

type EvalResult = Result<Literal, RuntimeError>;

impl Visitor<EvalResult> for Interpreter {
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
                Ok(Literal::Number(left / right))
            }

            TokenType::Star => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Number(left * right))
            }

            TokenType::Plus => match (left, right) {
                (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Number(left + right)),
                (Literal::String(left), Literal::String(right)) => Ok(Literal::String(left + &right)),
                _ => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::Greater => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Boolean(left > right))
            }

            TokenType::GreaterEqual => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Boolean(left >= right))
            }

            TokenType::Less => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Boolean(left < right))
            }

            TokenType::LessEqual => {
                let (left, right) = check_number_operands(operator, &left, &right)?;
                Ok(Literal::Boolean(left <= right))
            }

            TokenType::BangEqual => Ok(Literal::Boolean(left.is_truthy() != left.is_truthy())),

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
        todo!()
    }

    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> EvalResult {
        todo!()
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
