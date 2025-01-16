use anyhow::Result;
use std::{fs::File, io::Read};

mod generator;
mod ir;
mod parser;

fn main() -> Result<()> {
    if let Some(path) = std::env::args().nth(1) {
        let mut s = String::new();
        File::open(path)?.read_to_string(&mut s)?;

        match parser::parse(&s) {
            Ok(doc) => println!("{doc}"),
            Err(err) => eprintln!("{err}"),
        }
    }

    Ok(())
}
