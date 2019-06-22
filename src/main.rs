use crate::core::Scanner;
use std::io;

mod core;

// These constants will be provided by the CLI later.
const DIR: &str = "./";

fn main() -> Result<(), io::Error> {
    let scanner = Scanner{};
    scanner.run(DIR)
}