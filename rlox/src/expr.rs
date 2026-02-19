use crate::token::{self, Token};

#[derive(Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(Token, Box<Expr>),
    Comma(Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
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

    pub fn ternary(cond: Expr, left: Expr, right: Expr) -> Expr {
        Expr::Ternary(Box::new(cond), Box::new(left), Box::new(right))
    }

    pub fn assign(name: Token, value: Expr) -> Expr {
        Expr::Assign(name, Box::new(value))
    }

    pub fn logical(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Logical(Box::new(left), operator, Box::new(right))
    }

    pub fn call(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Expr {
        Expr::Call(Box::new(callee), paren, arguments)
    }
}

pub trait Visitor<R> {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_literal(&mut self, literal: &token::Literal) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> R;
    fn visit_comma(&mut self, left: &Expr, right: &Expr) -> R;
    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> R;
    fn visit_variable(&mut self, name: &Token) -> R;
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> R;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> R;
}

impl Expr {
    pub fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Expr::Grouping(expression) => visitor.visit_grouping(expression),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Comma(left, right) => visitor.visit_comma(left, right),
            Expr::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expr::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            Expr::Logical(left, operator, right) => visitor.visit_logical(left, operator, right),
            Expr::Ternary(cond, left, right) => visitor.visit_ternary(cond, left, right),
            Expr::Variable(name) => visitor.visit_variable(name),
            Expr::Assign(name, value) => visitor.visit_assign(name, value),
            Expr::Call(callee, paren, arguments) => visitor.visit_call(callee, paren, arguments),
        }
    }
}
