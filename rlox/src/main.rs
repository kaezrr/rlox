use std::path::Path;

use rlox::Lox;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut lox = Lox::default();

    match args.len() {
        0 => lox.run_prompt(),
        1 => lox.run_file(Path::new(&args[0])),
        _ => eprintln!("Usage: rlox [script]"),
    };
}
