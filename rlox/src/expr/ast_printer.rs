use super::*;

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

impl Visitor<String> for AstPrinter {
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
}
