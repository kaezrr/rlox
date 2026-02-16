use crate::{
    expr::Expr,
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    errors: Vec<ParseError>,
    current: usize,
}

impl<'a> Parser<'a> {
    // TODO: make this store a token
    fn error(&self, err_msg: &str) -> ParseError {
        ParseError {
            token_index: self.current,
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

    pub fn parse(mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        let next_tokens_to_match = [TokenType::BangEqual, TokenType::EqualEqual];

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        let next_tokens_to_match = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        let next_tokens_to_match = [TokenType::Minus, TokenType::Plus];

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        let next_tokens_to_match = [TokenType::Slash, TokenType::Star];

        while self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let next_tokens_to_match = [TokenType::Bang, TokenType::Minus];

        if self.advance_if(&next_tokens_to_match) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::unary(operator, right));
        }

        self.primary()
    }

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

    fn consume(&mut self, token_type: TokenType, err_msg: &str) -> Result<&Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(err_msg))
    }

    fn advance_if(&mut self, token_types: &[TokenType]) -> bool {
        for &token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
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

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub token_index: usize,
    pub message: String,
}
