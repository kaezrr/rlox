use crate::{expr::Expr, token::Token};

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Return(Token, Option<Expr>),
    Break,
}

impl Stmt {
    pub fn if_else(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Stmt {
        Stmt::If(condition, Box::new(then_branch), else_branch.map(Box::new))
    }

    pub fn while_st(condition: Expr, body: Stmt) -> Stmt {
        Stmt::While(condition, Box::new(body))
    }
}

pub trait Visitor<R> {
    fn visit_print_stmt(&mut self, expr: &Expr) -> R;
    fn visit_expression_stmt(&mut self, expr: &Expr) -> R;
    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> R;
    fn visit_block(&mut self, stmts: &[Stmt]) -> R;
    fn visit_if_else(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> R;
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> R;
    fn visit_break(&mut self) -> R;
    fn visit_return(&mut self, keyword: &Token, value: Option<&Expr>) -> R;
}

impl Stmt {
    pub fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr),
            Stmt::Print(expr) => visitor.visit_print_stmt(expr),
            Stmt::Var(name, initializer) => visitor.visit_var_stmt(name, initializer.as_ref()),
            Stmt::Block(stmts) => visitor.visit_block(stmts),
            Stmt::If(cond, then_b, else_b) => visitor.visit_if_else(cond, then_b, else_b.as_deref()),
            Stmt::While(condition, body) => visitor.visit_while(condition, body),
            Stmt::Break => visitor.visit_break(),
            Stmt::Return(keyword, value) => visitor.visit_return(keyword, value.as_ref()),
        }
    }
}
