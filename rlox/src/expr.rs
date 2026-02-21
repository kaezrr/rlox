use crate::{
    stmt::Stmt,
    token::{self, Token},
};

pub type ExprId = u32;

#[derive(Clone, Debug)]
pub struct Expr {
    pub id: ExprId,
    pub kind: ExprKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LambdaType {
    ClassStatic,
    Function,
    Getter,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(Token, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Lambda(Option<Token>, Vec<Token>, Vec<Stmt>, LambdaType),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    This(Token),
    Super(Token, Token),
    List(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>, Token),
    IndexSet(Box<Expr>, Box<Expr>, Box<Expr>, Token),
}

impl ExprKind {
    pub fn binary(left: Expr, operator: Token, right: Expr) -> ExprKind {
        ExprKind::Binary(Box::new(left), operator, Box::new(right))
    }

    pub fn unary(operator: Token, right: Expr) -> ExprKind {
        ExprKind::Unary(operator, Box::new(right))
    }

    pub fn literal(literal: token::Literal) -> ExprKind {
        ExprKind::Literal(literal)
    }

    pub fn grouping(expr: Expr) -> ExprKind {
        ExprKind::Grouping(Box::new(expr))
    }

    pub fn ternary(cond: Expr, left: Expr, right: Expr) -> ExprKind {
        ExprKind::Ternary(Box::new(cond), Box::new(left), Box::new(right))
    }

    pub fn assign(name: Token, value: Expr) -> ExprKind {
        ExprKind::Assign(name, Box::new(value))
    }

    pub fn logical(left: Expr, operator: Token, right: Expr) -> ExprKind {
        ExprKind::Logical(Box::new(left), operator, Box::new(right))
    }

    pub fn call(callee: Expr, paren: Token, arguments: Vec<Expr>) -> ExprKind {
        ExprKind::Call(Box::new(callee), paren, arguments)
    }

    pub fn get(object: Expr, name: Token) -> ExprKind {
        ExprKind::Get(Box::new(object), name)
    }

    pub fn set(object: Box<Expr>, name: Token, value: Expr) -> ExprKind {
        ExprKind::Set(object, name, Box::new(value))
    }

    pub fn index(list: Expr, index: Expr, paren: Token) -> ExprKind {
        ExprKind::Index(Box::new(list), Box::new(index), paren)
    }

    pub fn index_set(list: Box<Expr>, index: Box<Expr>, value: Expr, paren: Token) -> ExprKind {
        ExprKind::IndexSet(list, index, Box::new(value), paren)
    }
}

pub trait Visitor<R> {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_literal(&mut self, literal: &token::Literal) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> R;
    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> R;
    fn visit_variable(&mut self, name: &Token, expr: &Expr) -> R;
    fn visit_assign(&mut self, name: &Token, expr: &Expr, value: &Expr) -> R;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> R;
    fn visit_lambda(&mut self, name: Option<&Token>, params: &[Token], body: &[Stmt]) -> R;
    fn visit_get(&mut self, object: &Expr, name: &Token) -> R;
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> R;
    fn visit_this(&mut self, keyword: &Token, expr: &Expr) -> R;
    fn visit_super(&mut self, keyword: &Token, method: &Token, expr: &Expr) -> R;
    fn visit_list(&mut self, exprs: &[Expr]) -> R;
    fn visit_index(&mut self, list: &Expr, index: &Expr, paren: &Token) -> R;
    fn visit_index_set(&mut self, list: &Expr, index: &Expr, value: &Expr, paren: &Token) -> R;
}

impl Expr {
    pub fn accept<R, V: Visitor<R>>(&self, visitor: &mut V) -> R {
        match &self.kind {
            ExprKind::Grouping(expression) => visitor.visit_grouping(expression),
            ExprKind::Literal(literal) => visitor.visit_literal(literal),
            ExprKind::Unary(operator, right) => visitor.visit_unary(operator, right),
            ExprKind::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            ExprKind::Logical(left, operator, right) => visitor.visit_logical(left, operator, right),
            ExprKind::Ternary(cond, left, right) => visitor.visit_ternary(cond, left, right),
            ExprKind::Variable(name) => visitor.visit_variable(name, self),
            ExprKind::Assign(name, value) => visitor.visit_assign(name, self, value),
            ExprKind::Call(callee, paren, arguments) => visitor.visit_call(callee, paren, arguments),
            ExprKind::Lambda(name, params, body, _) => visitor.visit_lambda(name.as_ref(), params, body),
            ExprKind::Get(object, name) => visitor.visit_get(object, name),
            ExprKind::Set(object, name, value) => visitor.visit_set(object, name, value),
            ExprKind::This(keyword) => visitor.visit_this(keyword, self),
            ExprKind::Super(keyword, method) => visitor.visit_super(keyword, method, self),
            ExprKind::List(exprs) => visitor.visit_list(exprs),
            ExprKind::Index(list, index, paren) => visitor.visit_index(list, index, paren),
            ExprKind::IndexSet(list, index, value, paren) => visitor.visit_index_set(list, index, value, paren),
        }
    }
}
