use super::*;

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

impl Visitor<String> for RpnPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.post_orderize(&operator.lexeme, &[left, right])
    }

    fn visit_grouping(&mut self, expression: &Expr) -> String {
        expression.accept(self)
    }

    fn visit_literal(&mut self, literal: &token::Literal) -> String {
        literal.value()
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
}
