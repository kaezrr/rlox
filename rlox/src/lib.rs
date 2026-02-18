mod callable;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod stmt;
mod token;

use std::{io::Write, path::Path};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::{ParseError, Parser},
    token::{Scanner, Token, TokenType},
};

#[derive(Default)]
pub struct Lox {
    interpreter: Interpreter,

    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn run_file(&mut self, path: &Path) {
        let lines = std::fs::read_to_string(path).expect("read file");

        self.run::<false>(lines);

        if self.had_error {
            std::process::exit(65);
        }

        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    pub fn run_prompt(&mut self) {
        loop {
            print!("> ");
            std::io::stdout().flush().expect("flush io");

            let line = read_line();
            if line.is_empty() {
                break;
            }

            self.run::<true>(line);

            self.had_error = false;
        }
    }

    fn run<const IS_REPL: bool>(&mut self, source: String) {
        let (tokens, scan_errors) = Scanner::new(source).scan_tokens();
        for error in scan_errors {
            self.report(error.line, "", &error.message);
        }

        if IS_REPL {
            self.run_repl(&tokens);
        } else {
            self.run_script(&tokens);
        }
    }

    /// REPL mode supports running and printing single expressions
    fn run_repl(&mut self, tokens: &[Token]) {
        let (statements, parse_errors) = Parser::new(tokens).parse();
        if parse_errors.is_empty() {
            if let Err(e) = self.interpreter.interpret(&statements) {
                self.report_runtime_error(e);
            }
            return;
        }

        match Parser::new(tokens).parse_expression() {
            Ok(expr) => match self.interpreter.evaluate(&expr) {
                Ok(value) => eprintln!("{value}"),
                Err(e) => self.report_runtime_error(e),
            },
            Err(_) => {
                // Original statement errors are more meaningful
                for err in parse_errors {
                    self.report_parse_error(err);
                }
            }
        }
    }

    fn run_script(&mut self, tokens: &[Token]) {
        let (statements, parse_errors) = Parser::new(tokens).parse();
        if !parse_errors.is_empty() {
            for err in parse_errors {
                self.report_parse_error(err);
            }
            return;
        }

        if let Err(e) = self.interpreter.interpret(&statements) {
            self.report_runtime_error(e);
        };
    }

    fn report_parse_error(&mut self, err: ParseError) {
        let token = err.token;
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", &err.message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), &err.message);
        }
    }

    fn report_runtime_error(&mut self, err: RuntimeError) {
        eprintln!("{}\n[line {}]", err.message, err.token.line);
        self.had_runtime_error = true;
    }

    fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {line}] Error{place}: {message}");
        self.had_error = true;
    }
}

fn read_line() -> String {
    let mut read_line = String::new();
    std::io::stdin().read_line(&mut read_line).expect("read line");
    read_line
}
