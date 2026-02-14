use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.len() {
        0 => rlox::run_prompt(),
        1 => rlox::run_file(Path::new(&args[0])),
        _ => Err("Usage: rlox [script]".into()),
    }
}
