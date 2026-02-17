use crate::{expr::Expr, token::Token};

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Expr),
}

pub trait Visitor<R> {
    fn visit_print_stmt(&mut self, expr: &Expr) -> R;
    fn visit_expression_stmt(&mut self, expr: &Expr) -> R;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> R;
}

impl Stmt {
    pub fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr),
            Stmt::Print(expr) => visitor.visit_print_stmt(expr),
            Stmt::Var(name, initializer) => visitor.visit_var_stmt(name, initializer),
        }
    }
}
