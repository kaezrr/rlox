use std::{fmt::Display, rc::Rc};

use crate::{callable::Callable, class::LoxInstance};

pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    current: usize,
    line: usize,

    tokens: Vec<Token>,
    errors: Vec<ScanError>,
}

pub struct ScanError {
    pub line: usize,
    pub message: String,
}

impl Scanner {
    fn error(&mut self, line: usize, message: &str) {
        self.errors.push(ScanError {
            line,
            message: message.to_string(),
        });
    }

    pub fn new(source: String) -> Self {
        Self {
            source: source.as_bytes().to_owned(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Consumes scanner and returns created tokens and any accompanying errors
    pub fn scan_tokens(mut self) -> (Vec<Token>, Vec<ScanError>) {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        // End of file token
        self.tokens.push(Token::new(TokenType::Eof, "".into(), None, self.line));

        (self.tokens, self.errors)
    }

    fn scan_token(&mut self) {
        macro_rules! two_char {
            ($ch:expr, $two:expr, $one:expr) => {{
                let token = if self.advance_if($ch) { $two } else { $one };
                self.add_token(token);
            }};
        }

        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '?' => self.add_token(TokenType::Question),
            ':' => self.add_token(TokenType::Colon),

            '=' => two_char!('=', TokenType::EqualEqual, TokenType::Equal),
            '!' => two_char!('=', TokenType::BangEqual, TokenType::Bang),
            '<' => two_char!('=', TokenType::LessEqual, TokenType::Less),
            '>' => two_char!('=', TokenType::GreaterEqual, TokenType::Greater),

            '/' => {
                if self.advance_if('/') {
                    // Double slash indicates comments
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.advance_if('*') {
                    // Slash star means multi-line comment
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_next() == '/' {
                            break;
                        }

                        if self.peek() == '\n' {
                            self.line += 1;
                        }

                        self.advance();
                    }

                    if self.is_at_end() {
                        self.error(self.line, "Unterminated multi-line comment");
                        return;
                    }

                    // Consume the terminating star and slash
                    self.advance();
                    self.advance();
                } else {
                    self.add_token(TokenType::Slash)
                }
            }

            // Ignore whitespace
            ' ' | '\r' | '\t' => (),

            '\n' => self.line += 1,

            '"' => self.string(),

            ch => {
                if ch.is_ascii_digit() {
                    self.number();
                } else if is_alpha(ch) {
                    self.identifier();
                } else {
                    self.error(self.line, "Unexpected character");
                }
            }
        }
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let bytes = &self.source[self.start..self.current];
        let text = unsafe { str::from_utf8_unchecked(bytes) };
        let token_type = try_keyword(text).unwrap_or(TokenType::Identifier);

        match token_type {
            TokenType::True => self.add_token_literal(TokenType::True, Some(Literal::Boolean(true))),
            TokenType::False => self.add_token_literal(TokenType::False, Some(Literal::Boolean(false))),
            TokenType::Nil => self.add_token_literal(TokenType::Nil, Some(Literal::Nil)),
            t => self.add_token(t),
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let bytes = &self.source[self.start..self.current];
        let number = unsafe { str::from_utf8_unchecked(bytes) }.parse().expect("Parse f64");

        self.add_token_literal(TokenType::Number, Some(Literal::Number(number)));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string");
            return;
        }

        // The closing ".
        self.advance();

        let bytes = &self.source[(self.start + 1)..(self.current - 1)];
        let value = unsafe { str::from_utf8_unchecked(bytes) }.to_string();

        self.add_token_literal(TokenType::String, Some(Literal::String(value)));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current].into();
        self.current += 1;
        ch
    }

    fn advance_if(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None);
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let bytes = self.source[self.start..self.current].to_owned();
        let lexeme = unsafe { String::from_utf8_unchecked(bytes) };

        let token = Token::new(token_type, lexeme, literal, self.line);
        self.tokens.push(token);
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current].into()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1].into()
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || c.is_ascii_digit()
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.literal {
            Some(lit) => write!(f, "{:?} {} {}", self.token_type, self.lexeme, lit),
            None => write!(f, "{:?} {}", self.token_type, self.lexeme),
        }
    }
}

fn try_keyword(keyword_str: &str) -> Option<TokenType> {
    match keyword_str {
        "and" => Some(TokenType::And),
        "class" => Some(TokenType::Class),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "for" => Some(TokenType::For),
        "fun" => Some(TokenType::Fun),
        "if" => Some(TokenType::If),
        "nil" => Some(TokenType::Nil),
        "or" => Some(TokenType::Or),
        "print" => Some(TokenType::Print),
        "return" => Some(TokenType::Return),
        "super" => Some(TokenType::Super),
        "this" => Some(TokenType::This),
        "true" => Some(TokenType::True),
        "var" => Some(TokenType::Var),
        "while" => Some(TokenType::While),
        "break" => Some(TokenType::Break),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single character
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,
    Colon,

    // One or two characters
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,

    // Literals
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,

    Eof,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Callable(Rc<Callable>),
    Instance(Rc<LoxInstance>),
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(v) => write!(f, "{}", v),
            Literal::String(v) => write!(f, "\"{}\"", v),
            Literal::Boolean(v) => write!(f, "{}", v),
            Literal::Nil => write!(f, "nil"),
            Literal::Callable(v) => write!(f, "{}", v.name),
            Literal::Instance(v) => write!(f, "{}", v),
        }
    }
}

impl Literal {
    /// Lox follows Ruby’s simple rule: false and nil are falsey, and everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::Boolean(v) => *v,
            _ => true,
        }
    }
}
