use std::fmt::Display;

pub struct Scanner<'a> {
    source: Vec<u8>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    err_fn: ErrorHandler<'a>,
}

type ErrorHandler<'a> = Box<dyn FnMut(usize, &str) + 'a>;

impl<'a> Scanner<'a> {
    fn error(&mut self, line: usize, message: &str) {
        (self.err_fn)(line, message);
    }

    pub fn new(source: String, err_fn: ErrorHandler<'a>) -> Self {
        Self {
            source: source.as_bytes().to_owned(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            err_fn,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        let end_of_file = Token::new(TokenType::Eof, "".into(), None, self.line);
        self.tokens.push(end_of_file);

        &self.tokens
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
        self.add_token(try_keyword(text).unwrap_or(TokenType::Identifier));
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
        let number = unsafe { str::from_utf8_unchecked(bytes) }
            .parse()
            .expect("Parse f64");

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

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
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
            Some(lit) => write!(f, "{:?} {} {:?}", self.token_type, self.lexeme, lit),
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
        _ => None,
    }
}

#[derive(Debug)]
enum TokenType {
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

    Eof,
}

#[derive(Debug, PartialEq)]
enum Literal {
    Number(f64),
    String(String),
}
