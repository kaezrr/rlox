mod ast_printer;
mod rpn_printer;

pub use ast_printer::AstPrinter;
pub use rpn_printer::RpnPrinter;

use crate::token::{self, Token};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(Token, Box<Expr>),
}

impl Expr {
    pub fn binary(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Binary(Box::new(left), operator, Box::new(right))
    }

    pub fn unary(operator: Token, right: Expr) -> Expr {
        Expr::Unary(operator, Box::new(right))
    }

    pub fn literal(literal: token::Literal) -> Expr {
        Expr::Literal(literal)
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(expr))
    }
}

trait Visitor<R> {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_literal(&mut self, literal: &token::Literal) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> R;
}

impl Expr {
    fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Expr::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            Expr::Grouping(expression) => visitor.visit_grouping(expression),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(operator, right) => visitor.visit_unary(operator, right),
        }
    }
}
