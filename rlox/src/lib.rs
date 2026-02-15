mod expr;
mod token;

use std::{io::Write, path::Path};

use crate::token::Scanner;

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
        let mut scanner = Scanner::new(
            source,
            Box::new(|line, message| {
                self.error(line, message);
            }),
        );

        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!("{token}");
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {line}] Error {place}: {message}");
        self.had_error = true;
    }
}

fn read_line() -> String {
    let mut read_line = String::new();
    std::io::stdin()
        .read_line(&mut read_line)
        .expect("read line");
    read_line
}
