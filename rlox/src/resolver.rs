use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{
    expr::{self, Expr, ExprKind, LambdaType},
    interpreter::Interpreter,
    stmt::{self, Stmt},
    token::{Literal, Token, TokenType},
};

#[derive(Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method(Token),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ClassType {
    None,
    Class,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum VarState {
    Unintialized,
    Unused,
    Used,
}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lexeme.hash(state);
    }
}
impl Eq for Token {}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.lexeme == other.lexeme
    }
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<Token, VarState>>,
    errors: Vec<ResolveError>,
    current_function: FunctionType,
    current_class: ClassType,
    in_loop: bool,
    in_static: bool,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
            errors: Default::default(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            in_loop: false,
            in_static: false,
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
        let Some(popped_scope) = self.scopes.pop() else {
            return;
        };

        for (k, v) in popped_scope {
            if v == VarState::Unused {
                self.error(&k, "Unused local variable.");
            }
        }
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

        if scope.contains_key(name) {
            self.error(name, "Already a variable with this name in this scope.");
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), VarState::Unintialized);
    }

    fn define(&mut self, name: &Token) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        *scope.get_mut(name).unwrap() = VarState::Unused;
    }

    fn resolve_local(&mut self, name: &Token, expr: &Expr) {
        for (i, scope) in self.scopes.iter_mut().enumerate().rev() {
            if let Some(key) = scope.get_mut(name) {
                *key = VarState::Used;
                self.interpreter.resolve(expr, self.scopes.len() - i - 1);
                return;
            }
        }
    }

    fn resolve_lambda(&mut self, params: &[Token], body: &[Stmt], ftype: FunctionType) {
        let enclosing_function = std::mem::replace(&mut self.current_function, ftype);

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
            && matches!(last.get(name), Some(VarState::Unintialized))
        {
            self.error(name, "Can't use uninitialized variable.");
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

    fn visit_get(&mut self, object: &Expr, _name: &Token) {
        self._resolve(object);
    }

    fn visit_set(&mut self, object: &Expr, _name: &Token, value: &Expr) {
        self._resolve(value);
        self._resolve(object);
    }

    fn visit_this(&mut self, keyword: &Token, expr: &Expr) {
        if self.current_class == ClassType::None {
            return self.error(keyword, "Can't use 'this' outside of a class.");
        }

        if self.in_static {
            return self.error(keyword, "Can't use 'this' inside static functions.");
        }

        self.resolve_local(keyword, expr);
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
            self.define(name);
        }
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
        let enclosed_loop = self.in_loop;
        self.in_loop = true;

        self._resolve(condition);
        self._resolve(body);

        self.in_loop = enclosed_loop;
    }

    fn visit_break(&mut self, keyword: &Token) {
        if !self.in_loop {
            self.error(keyword, "Can't break outside loops.");
        }
    }

    fn visit_return(&mut self, keyword: &Token, value: Option<&Expr>) {
        if self.current_function == FunctionType::None {
            self.error(keyword, "Can't return from top-level code");
        }

        if let Some(value) = value {
            if self.current_function == FunctionType::Initializer {
                self.error(keyword, "Can't return a value from an initializer.");
            }
            self._resolve(value);
        }
    }

    fn visit_class(&mut self, name: &Token, methods: &[Stmt]) {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(name);
        self.define(name);

        self.begin_scope();

        // Define this
        self.scopes.last_mut().unwrap().insert(
            Token::new(TokenType::This, "this".to_string(), None, name.line),
            VarState::Used,
        );

        for m in methods {
            let Stmt::Var(name, Some(expr)) = m else {
                return self.error(name, "Only methods allowed in class body.");
            };

            if let ExprKind::Lambda(_, params, body, lambda_type) = &expr.kind {
                let ftype = if name.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method(name.clone())
                };

                let previous = self.in_static;
                self.in_static = *lambda_type == LambdaType::ClassStatic;

                self.resolve_lambda(params, body, ftype);

                self.in_static = previous;
            } else {
                return self.error(name, "Only methods allowed in class body.");
            }
        }

        self.end_scope();

        self.current_class = enclosing_class;
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
