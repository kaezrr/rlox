use std::{collections::HashMap, fmt::Debug};

use crate::{
    expr::{self, Expr},
    interpreter::Interpreter,
    stmt::{self, Stmt},
    token::{Literal, Token},
};

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolveError>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
            errors: Default::default(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve(mut self, stmts: &[Stmt]) -> Vec<ResolveError> {
        self._resolve(stmts);
        self.errors
    }

    fn _resolve<R: Resolve + ?Sized + Debug>(&mut self, thing: &R) {
        thing.resolve(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn error(&mut self, name: &Token, message: &str) {
        self.errors.push(ResolveError {
            token: name.clone(),
            message: message.to_string(),
        });
    }

    fn declare(&mut self, name: &Token) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };
        if scope.contains_key(&name.lexeme) {
            self.error(name, "Already a variable with this name in this scope.");
        }
        self.scopes.last_mut().unwrap().insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, name: &Token, expr: &Expr) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - i - 1);
                return;
            }
        }
    }

    fn resolve_lambda(&mut self, params: &[Token], body: &[Stmt], ftype: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = ftype;

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self._resolve(body);
        self.end_scope();

        self.current_function = enclosing_function;
    }
}

impl expr::Visitor<()> for Resolver<'_> {
    fn visit_binary(&mut self, left: &Expr, _operator: &Token, right: &Expr) {
        self._resolve(left);
        self._resolve(right);
    }

    fn visit_logical(&mut self, left: &Expr, _operator: &Token, right: &Expr) {
        self._resolve(left);
        self._resolve(right);
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        self._resolve(expression);
    }

    fn visit_literal(&mut self, _literal: &Literal) {}

    fn visit_unary(&mut self, _operator: &Token, right: &Expr) {
        self._resolve(right);
    }

    fn visit_comma(&mut self, left: &Expr, right: &Expr) {
        self._resolve(left);
        self._resolve(right);
    }

    fn visit_ternary(&mut self, cond: &Expr, left: &Expr, right: &Expr) {
        self._resolve(cond);
        self._resolve(left);
        self._resolve(right);
    }

    fn visit_variable(&mut self, name: &Token, expr: &Expr) {
        if let Some(last) = self.scopes.last()
            && last.get(&name.lexeme).map(|x| !x).unwrap_or_default()
        {
            self.error(name, "Can't read local variable in its own initializer.");
        }

        self.resolve_local(name, expr);
    }

    fn visit_assign(&mut self, name: &Token, expr: &Expr, value: &Expr) {
        self._resolve(value);
        self.resolve_local(name, expr);
    }

    fn visit_call(&mut self, callee: &Expr, _paren: &Token, arguments: &[Expr]) {
        self._resolve(callee);
        for arg in arguments {
            self._resolve(arg);
        }
    }

    fn visit_lambda(&mut self, _name: Option<&Token>, params: &[Token], body: &[Stmt]) {
        self.resolve_lambda(params, body, FunctionType::Function);
    }
}

impl stmt::Visitor<()> for Resolver<'_> {
    fn visit_print_stmt(&mut self, expr: &Expr) {
        self._resolve(expr);
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) {
        self._resolve(expr);
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) {
        self.declare(name);
        if let Some(init) = initializer {
            self._resolve(init);
        }
        self.define(name);
    }

    fn visit_block(&mut self, stmts: &[Stmt]) {
        self.begin_scope();
        self._resolve(stmts);
        self.end_scope();
    }

    fn visit_if_else(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) {
        self._resolve(condition);
        self._resolve(then_branch);
        if let Some(else_branch) = else_branch {
            self._resolve(else_branch);
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) {
        self._resolve(condition);
        self._resolve(body);
    }

    fn visit_break(&mut self) {}

    fn visit_return(&mut self, keyword: &Token, value: Option<&Expr>) {
        if self.current_function == FunctionType::None {
            self.error(keyword, "Can't return from top-level code");
        }

        if let Some(value) = value {
            self._resolve(value);
        }
    }
}

trait Resolve {
    fn resolve(&self, resolver: &mut Resolver);
}

impl Resolve for Stmt {
    fn resolve(&self, resolver: &mut Resolver) {
        self.accept(resolver)
    }
}

impl Resolve for Expr {
    fn resolve(&self, resolver: &mut Resolver) {
        self.accept(resolver)
    }
}

impl Resolve for [Stmt] {
    fn resolve(&self, resolver: &mut Resolver) {
        for stmt in self {
            stmt.accept(resolver);
        }
    }
}

pub struct ResolveError {
    pub token: Token,
    pub message: String,
}
