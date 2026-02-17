use crate::{
    expr::Expr,
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    errors: Vec<ParseError>,
    current: usize,
}

impl<'a> Parser<'a> {
    fn error(&self, err_msg: &str) -> ParseError {
        ParseError {
            token: self.peek().clone(),
            message: err_msg.to_string(),
        }
    }

    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            errors: Vec::new(),
            current: 0,
        }
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

    /// declaration -> varDecl | statement
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.advance_if(&[TokenType::Var]) {
            return self.var_declaration();
        }

        self.statement()
    }

    /// varDecl -> "var" IDENTIFIER ("=" expression)? ";"
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?.clone();

        // Uninitialized variable
        let mut expression = Expr::Literal(Literal::Nil);
        if self.advance_if(&[TokenType::Equal]) {
            expression = self.expression()?;
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var(name, expression))
    }

    /// statement -> printStmt | exprStmt
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.advance_if(&[TokenType::Print]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    /// printStmt -> "print" expression ";"
    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    /// exprStmt -> expression ";"
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    /// expression -> comma
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.comma()
    }

    /// comma -> equality ("," equality)*
    fn comma(&mut self) -> Result<Expr, ParseError> {
        // Missing left operand
        if self.check(&[TokenType::Comma]) {
            let err = self.error("Expect expression before comma");
            while self.advance_if(&[TokenType::Comma]) {
                let _ = self.ternary();
            }
            return Err(err);
        }

        let mut expr = self.ternary()?;

        while self.advance_if(&[TokenType::Comma]) {
            let right = self.ternary()?;
            expr = Expr::comma(expr, right);
        }

        Ok(expr)
    }

    /// ternary -> (equality "?" ternary ":" ternary) | equality
    fn ternary(&mut self) -> Result<Expr, ParseError> {
        // Missing left operand
        if self.check(&[TokenType::Question]) {
            let err = self.error("Expect condition in ternary expression");
            self.advance();
            let _ = self.ternary();
            let _ = self.ternary();
            return Err(err);
        }

        let mut expr = self.equality()?;

        if self.advance_if(&[TokenType::Question]) {
            let left = self.ternary()?;
            self.consume(TokenType::Colon, "Expect ':' after ternary expression")?;
            let right = self.ternary()?;
            expr = Expr::ternary(expr, left, right);
        }

        Ok(expr)
    }

    /// equality -> comparison (("==" | "!=") comparison)*
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [TokenType::BangEqual, TokenType::EqualEqual];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error("Expect expression before equality");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.comparison();
            }
            return Err(err);
        }

        let mut expr = self.comparison()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    /// comparison -> term (("> | ">=" | "<" | "<=") term)*
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error("Expect expression before comparison");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.term();
            }
            return Err(err);
        }

        let mut expr = self.term()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    /// term -> factor (("-" | "+") factor)*
    fn term(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [TokenType::Minus, TokenType::Plus];

        // Missing left operand
        if self.check(&[TokenType::Plus]) {
            let err = self.error("Expect expression before binary operator");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.factor();
            }
            return Err(err);
        }

        let mut expr = self.factor()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    /// factor -> unary (("/" | "*") unary)*
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [TokenType::Slash, TokenType::Star];

        // Missing left operand
        if self.check(&next_tokens_to_match) {
            let err = self.error("Expect expression before binary operator");
            while self.advance_if(&next_tokens_to_match) {
                let _ = self.unary();
            }
            return Err(err);
        }

        let mut expr = self.unary()?;

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    /// unary -> ("!" | "-") unary | primary
    fn unary(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [TokenType::Bang, TokenType::Minus];

        if self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::unary(operator, right));
        }

        self.primary()
    }

    /// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER
    fn primary(&mut self) -> Result<Expr, ParseError> {
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
            return Ok(Expr::literal(literal.clone()));
        }

        if self.advance_if(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::grouping(expr));
        }

        if self.advance_if(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        Err(self.error("Expect expression."))
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

        Err(self.error(err_msg))
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
