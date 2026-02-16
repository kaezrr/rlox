mod expr;
mod parser;
mod token;

use std::{io::Write, path::Path};

use crate::{
    expr::AstPrinter,
    parser::Parser,
    token::{Scanner, Token, TokenType},
};

#[derive(Default)]
pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn run_file(&mut self, path: &Path) {
        let lines = std::fs::read_to_string(path).expect("read file");

        self.run(lines);

        if self.had_error {
            std::process::exit(65);
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

            self.run(line);

            self.had_error = false;
        }
    }

    fn run(&mut self, source: String) {
        let (tokens, scan_errors) = Scanner::new(source).scan_tokens();
        for error in scan_errors {
            self.report(error.line, "", &error.message);
        }

        let expression = Parser::new(&tokens).parse();
        match expression {
            Ok(expr) => println!("{}", AstPrinter.print(&expr)),
            Err(parse_error) => self.error(&tokens[parse_error.token_index], &parse_error.message),
        }
    }

    pub fn error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
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
