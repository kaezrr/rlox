mod token;

use std::{error::Error, io::Write, path::Path};

struct Lox;

impl Lox {
    pub fn run_file(path: &Path) -> Result<(), Box<dyn Error>> {
        let lines = std::fs::read_to_string(path)?;
        Lox::run(lines)
    }

    pub fn run_prompt() -> Result<(), Box<dyn Error>> {
        loop {
            print!("> ");
            std::io::stdout().flush()?;

            let mut read_line = String::new();
            std::io::stdin().read_line(&mut read_line)?;

            if read_line.is_empty() {
                return Ok(());
            }

            if let Err(err) = Lox::run(read_line) {
                eprintln!("{err}")
            }
        }
    }

    pub fn error(line: usize, message: &str) -> String {
        report(line, "", message)
    }
}

fn run(source: String) -> Result<(), Box<dyn Error>> {
    let tokens = source.lines();

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}

fn report(line: usize, place: &str, message: &str) -> String {
    format!("[line {line}] Error {place}: {message}")
}
