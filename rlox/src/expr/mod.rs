mod ast_printer;
mod rpn_printer;

pub use ast_printer::AstPrinter;

use crate::token::{self, Token};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(Token, Box<Expr>),
    Comma(Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
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

    pub fn comma(left: Expr, right: Expr) -> Expr {
        Expr::Comma(Box::new(left), Box::new(right))
    }
}

trait Visitor<R> {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_literal(&mut self, literal: &token::Literal) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> R;
    fn visit_comma(&mut self, left: &Expr, right: &Expr) -> R;
    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> R;
}

impl Expr {
    fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Expr::Grouping(expression) => visitor.visit_grouping(expression),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Comma(left, right) => visitor.visit_comma(left, right),
            Expr::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expr::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            Expr::Ternary(cond, left, right) => visitor.visit_ternary(cond, left, right),
        }
    }
}
