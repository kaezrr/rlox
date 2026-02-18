use crate::{
    expr::{self, Expr},
    token::{self, Token},
};

#[allow(unused)]
pub struct AstPrinter;

#[allow(unused)]
impl AstPrinter {
    /// Prints the Abstract Syntax Tree (AST) of a given expression
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();
        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            builder.push(' ');
            builder.push_str(&expr.accept(self));
        }

        builder.push(')');
        builder
    }
}

#[allow(unused)]
pub struct RpnPrinter;

#[allow(unused)]
impl RpnPrinter {
    /// Prints the Reverse Polish Notation of a given expression
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn post_orderize(&mut self, op: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();

        for expr in exprs {
            builder.push_str(&expr.accept(self));
            builder.push(' ');
        }

        builder.push_str(op);
        builder
    }
}

impl expr::Visitor<String> for AstPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.parenthesize(&operator.lexeme, &[left, right])
    }

    fn visit_grouping(&mut self, expression: &Expr) -> String {
        self.parenthesize("group", &[expression])
    }

    fn visit_literal(&mut self, literal: &token::Literal) -> String {
        literal.to_string()
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> String {
        self.parenthesize(&operator.lexeme, &[right])
    }

    fn visit_comma(&mut self, left: &Expr, right: &Expr) -> String {
        self.parenthesize(",", &[left, right])
    }

    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> String {
        self.parenthesize("?:", &[cond, left, right])
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        name.lexeme.clone()
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> String {
        self.parenthesize(&format!("= {}", name.lexeme), &[value])
    }

    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.parenthesize(&operator.lexeme, &[left, right])
    }
}

impl expr::Visitor<String> for RpnPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.post_orderize(&operator.lexeme, &[left, right])
    }

    fn visit_grouping(&mut self, expression: &Expr) -> String {
        expression.accept(self)
    }

    fn visit_literal(&mut self, literal: &token::Literal) -> String {
        literal.to_string()
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> String {
        self.post_orderize(&operator.lexeme, &[right])
    }

    fn visit_comma(&mut self, left: &Expr, right: &Expr) -> String {
        self.post_orderize(",", &[left, right])
    }

    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> String {
        self.post_orderize("?:", &[cond, left, right])
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        name.lexeme.clone()
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> String {
        self.post_orderize(&format!("= {}", name.lexeme), &[value])
    }

    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.post_orderize(&operator.lexeme, &[left, right])
    }
}
