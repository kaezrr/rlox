use crate::{
    expr::{Expr, ExprKind, LambdaType},
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

type ParseExprResult = Result<Expr, ParseError>;
type ParseStmtResult = Result<Stmt, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    errors: Vec<ParseError>,
    current: usize,
    id_seed: u32,
}

impl<'a> Parser<'a> {
    fn error(&self, token: Token, err_msg: &str) -> ParseError {
        ParseError {
            token,
            message: err_msg.to_string(),
        }
    }

    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            errors: Vec::new(),
            current: 0,
            id_seed: 0,
        }
    }

    pub fn build_expr(&mut self, kind: ExprKind) -> Expr {
        self.id_seed += 1;
        Expr { id: self.id_seed, kind }
    }

    pub fn parse_expression(&mut self) -> ParseExprResult {
        self.expression()
    }

    pub fn parse(mut self) -> (Vec<Stmt>, Vec<ParseError>) {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.try_program() {
                statements.push(stmt);
            }
        }

        (statements, self.errors)
    }

    fn try_program(&mut self) -> Option<Stmt> {
        match self.declaration() {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                self.errors.push(e);
                self.synchronize();
                None
            }
        }
    }

    /// declaration -> varDecl | funDecl | classDecl | statement
    fn declaration(&mut self) -> ParseStmtResult {
        if self.advance_if(&[TokenType::Var]) {
            return self.var_declaration();
        }

        if self.advance_if(&[TokenType::Fun]) {
            return self.function_declaration("function", LambdaType::Function);
        }

        if self.advance_if(&[TokenType::Class]) {
            return self.class_declaration();
        }

        self.statement()
    }

    /// "class" IDENTIFIER "{" (function | staticFunction| getter)* "}"
    fn class_declaration(&mut self) -> ParseStmtResult {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?.clone();
        self.consume(TokenType::LeftBrace, "Expect '{' after class body")?;

        let mut methods = Vec::new();
        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            if self.advance_if(&[TokenType::Class]) {
                methods.push(self.function_declaration("static method", LambdaType::ClassStatic)?);
            } else {
                methods.push(self.function_declaration("method or getter", LambdaType::Getter)?);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;

        Ok(Stmt::Class(name, methods))
    }

    /// funDecl -> lambda | "fun" IDENTIFIER "(" parameters? ")" block
    fn function_declaration(&mut self, kind: &str, lambda_type: LambdaType) -> ParseStmtResult {
        if self.advance_if(&[TokenType::Identifier]) {
            let name = self.previous().clone();
            let function = if lambda_type == LambdaType::Getter {
                if self.advance_if(&[TokenType::LeftBrace]) {
                    self.lambda(Some(name.clone()), "getter", LambdaType::Getter)
                } else {
                    self.lambda(Some(name.clone()), "method", LambdaType::Function)
                }
            } else {
                self.lambda(Some(name.clone()), kind, lambda_type)
            };

            return Ok(Stmt::Var(name, Some(function?)));
        }

        Ok(Stmt::Expression(self.lambda(None, kind, lambda_type)?))
    }

    /// function -> "fun" "(" parameters? ")" block
    fn lambda(&mut self, name: Option<Token>, kind: &str, lambda_type: LambdaType) -> ParseExprResult {
        if lambda_type == LambdaType::Getter {
            let body = self.block()?;
            return Ok(self.build_expr(ExprKind::Lambda(name, Vec::new(), body, lambda_type)));
        }

        self.consume(TokenType::LeftParen, &format!("Expect '(' after {}.", kind))?;
        let mut parameters = Vec::new();

        if !self.check(&[TokenType::RightParen]) {
            parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?.clone());
            while self.advance_if(&[TokenType::Comma]) {
                if parameters.len() >= 255 {
                    let err = self.error(self.peek().clone(), "Can't have more than 255 parameters.");
                    self.errors.push(err);
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?.clone());
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, &format!("Expect '{{' before {} body.", kind))?;

        let body = self.block()?;

        Ok(self.build_expr(ExprKind::Lambda(name, parameters, body, lambda_type)))
    }

    /// varDecl -> "var" IDENTIFIER ("=" expression)? ";"
    fn var_declaration(&mut self) -> ParseStmtResult {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?.clone();

        // Uninitialized variable
        let expression = if self.advance_if(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var(name, expression))
    }

    /// statement -> printStmt | ifStmt | exprStmt | whileStmt | forStmt | breakStmt | returnStmt | block
    fn statement(&mut self) -> ParseStmtResult {
        if self.advance_if(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.advance_if(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.advance_if(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.advance_if(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.advance_if(&[TokenType::Break]) {
            return self.break_statement();
        }

        if self.advance_if(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.advance_if(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        self.expression_statement()
    }

    /// returnStmt -> "return" expression? ";"
    fn return_statement(&mut self) -> ParseStmtResult {
        let keyword = self.previous().clone();
        let value = if !self.check(&[TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword, value))
    }

    /// breakStmt -> "break" ";"
    fn break_statement(&mut self) -> ParseStmtResult {
        let keyword = self.previous().clone();
        self.consume(TokenType::Semicolon, "Expect ';' after break.")?;
        Ok(Stmt::Break(keyword))
    }

    /// ifStmt -> "if" "(" expression ")" statement ("else" statement)?
    fn if_statement(&mut self) -> ParseStmtResult {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = if self.advance_if(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::if_else(condition, then_branch, else_branch))
    }

    /// block -> "{" declaration* "}"
    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    /// printStmt -> "print" expression ";"
    fn print_statement(&mut self) -> ParseStmtResult {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    /// exprStmt -> expression ";"
    fn expression_statement(&mut self) -> ParseStmtResult {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    /// whileStmt -> "while" "(" expression ")" statement
    fn while_statement(&mut self) -> ParseStmtResult {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let body = self.statement()?;

        Ok(Stmt::while_st(condition, body))
    }

    /// forStmt -> "for" "(" (varDel | exprStmt | ";") expression? ";" expression? ")" statement
    fn for_statement(&mut self) -> ParseStmtResult {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.advance_if(&[TokenType::Semicolon]) {
            None
        } else if self.advance_if(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&[TokenType::Semicolon]) {
            self.expression()?
        } else {
            self.build_expr(ExprKind::literal(Literal::Boolean(true)))
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }

        body = Stmt::while_st(condition, body);

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    /// expression -> comma
    fn expression(&mut self) -> ParseExprResult {
        self.comma()
    }

    /// comma -> assignment ("," assignment)*
    fn comma(&mut self) -> ParseExprResult {
        // Missing left operand
        if self.check(&[TokenType::Comma]) {
            let err = self.error(self.peek().clone(), "Expect expression before comma");
            while self.advance_if(&[TokenType::Comma]) {
                let _ = self.assignment();
            }
            return Err(err);
        }

        let mut expr = self.assignment()?;

        while self.advance_if(&[TokenType::Comma]) {
            let right = self.assignment()?;
            expr = self.build_expr(ExprKind::comma(expr, right));
        }

        Ok(expr)
    }

    /// assignment -> (call ".")? IDENTIFIER "=" assignment | ternary
    fn assignment(&mut self) -> ParseExprResult {
        let expr = self.ternary()?;

        if self.advance_if(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let ExprKind::Variable(name) = expr.kind {
                return Ok(self.build_expr(ExprKind::assign(name, value)));
            } else if let ExprKind::Get(object, name) = expr.kind {
                return Ok(self.build_expr(ExprKind::set(object, name, value)));
            }

            return Err(self.error(equals, "Invalid assignment target"));
        }

        Ok(expr)
    }

    /// ternary -> (logic_or "?" ternary ":" ternary) | logic_or
    fn ternary(&mut self) -> ParseExprResult {
        // Missing left operand
        if self.check(&[TokenType::Question]) {
            let err = self.error(self.peek().clone(), "Expect condition in ternary expression");
            self.advance();
            let _ = self.ternary();
            let _ = self.ternary();
            return Err(err);
        }

        let mut expr = self.or()?;

        if self.advance_if(&[TokenType::Question]) {
            let left = self.ternary()?;
            self.consume(TokenType::Colon, "Expect ':' after ternary expression")?;
            let right = self.ternary()?;
            expr = self.build_expr(ExprKind::ternary(expr, left, right));
        }

        Ok(expr)
    }

    /// logic_or = logic_and ("or" logic_and)*
    fn or(&mut self) -> ParseExprResult {
        let mut expr = self.and()?;

        while self.advance_if(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = self.build_expr(ExprKind::logical(expr, operator, right));
        }

        Ok(expr)
    }

    /// logic_and = equality ("and" equality)*
    fn and(&mut self) -> ParseExprResult {
        let mut expr = self.equality()?;

        while self.advance_if(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = self.build_expr(ExprKind::logical(expr, operator, right));
        }

        Ok(expr)
    }

    /// equality -> comparison (("==" | "!=") comparison)*
    fn equality(&mut self) -> ParseExprResult {
        let next_tokens_to_match = [TokenType::BangEqual, TokenType::EqualEqual];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error(self.peek().clone(), "Expect expression before equality");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.comparison();
            }
            return Err(err);
        }

        let mut expr = self.comparison()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = self.build_expr(ExprKind::binary(expr, operator, right));
        }

        Ok(expr)
    }

    /// comparison -> term (("> | ">=" | "<" | "<=") term)*
    fn comparison(&mut self) -> ParseExprResult {
        let next_tokens_to_match = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error(self.peek().clone(), "Expect expression before comparison");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.term();
            }
            return Err(err);
        }

        let mut expr = self.term()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = self.build_expr(ExprKind::binary(expr, operator, right));
        }

        Ok(expr)
    }

    /// term -> factor (("-" | "+") factor)*
    fn term(&mut self) -> ParseExprResult {
        let next_tokens_to_match = [TokenType::Minus, TokenType::Plus];

        // Missing left operand
        if self.check(&[TokenType::Plus]) {
            let err = self.error(self.peek().clone(), "Expect expression before binary operator");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.factor();
            }
            return Err(err);
        }

        let mut expr = self.factor()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = self.build_expr(ExprKind::binary(expr, operator, right));
        }

        Ok(expr)
    }

    /// factor -> unary (("/" | "*") unary)*
    fn factor(&mut self) -> ParseExprResult {
        let next_tokens_to_match = [TokenType::Slash, TokenType::Star];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error(self.peek().clone(), "Expect expression before binary operator");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.unary();
            }
            return Err(err);
        }

        let mut expr = self.unary()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = self.build_expr(ExprKind::binary(expr, operator, right));
        }

        Ok(expr)
    }

    /// unary -> ("!" | "-") unary | call
    fn unary(&mut self) -> ParseExprResult {
        let next_tokens_to_match = [TokenType::Bang, TokenType::Minus];

        if self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(self.build_expr(ExprKind::unary(operator, right)));
        }

        self.call()
    }

    /// call -> primary ( "(" arguments? ")" | "." IDENTIFIER )*
    fn call(&mut self) -> ParseExprResult {
        let mut expr = self.primary()?;

        loop {
            if self.advance_if(&[TokenType::LeftParen]) {
                expr = self.arguments(expr)?;
            } else if self.advance_if(&[TokenType::Dot]) {
                let name = self
                    .consume(TokenType::Identifier, "Expect property name after '.'.")?
                    .clone();
                expr = self.build_expr(ExprKind::get(expr, name))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// arguments -> assignment ("," assignment)*
    fn arguments(&mut self, callee: Expr) -> ParseExprResult {
        let mut arguments = Vec::new();
        if !self.check(&[TokenType::RightParen]) {
            arguments.push(self.assignment()?);

            while self.advance_if(&[TokenType::Comma]) {
                if arguments.len() >= 255 {
                    let err = self.error(self.peek().clone(), "Can't have more than 255 arguments.");
                    self.errors.push(err);
                }
                arguments.push(self.assignment()?);
            }
        }

        let paren = self
            .consume(TokenType::RightParen, "Expect ')' after arguments.")?
            .clone();

        Ok(self.build_expr(ExprKind::call(callee, paren, arguments)))
    }

    /// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER |
    /// Lambda
    fn primary(&mut self) -> ParseExprResult {
        // Literals
        let next_tokens_to_match = [
            TokenType::True,
            TokenType::False,
            TokenType::Nil,
            TokenType::Number,
            TokenType::String,
        ];

        if self.advance_if(&next_tokens_to_match) {
            let literal = self.previous().literal.as_ref().unwrap();
            return Ok(self.build_expr(ExprKind::literal(literal.clone())));
        }

        if self.advance_if(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(self.build_expr(ExprKind::grouping(expr)));
        }

        if self.advance_if(&[TokenType::This]) {
            return Ok(self.build_expr(ExprKind::This(self.previous().clone())));
        }

        if self.advance_if(&[TokenType::Identifier]) {
            return Ok(self.build_expr(ExprKind::Variable(self.previous().clone())));
        }

        if self.advance_if(&[TokenType::Fun]) {
            return self.lambda(None, "function", LambdaType::Function);
        }

        Err(self.error(self.peek().clone(), "Expect expression."))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// Consume a given token or return error if it doesn't exist
    fn consume(&mut self, token_type: TokenType, err_msg: &str) -> Result<&Token, ParseError> {
        if self.check(&[token_type]) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek().clone(), err_msg))
    }

    fn advance_if(&mut self, types: &[TokenType]) -> bool {
        if !self.check(types) {
            return false;
        }

        self.advance();
        true
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, types: &[TokenType]) -> bool {
        if self.is_at_end() {
            return false;
        }
        types.iter().any(|t| self.peek().token_type == *t)
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}

pub struct ParseError {
    pub token: Token,
    pub message: String,
}
