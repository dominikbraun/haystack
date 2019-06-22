use std::io;

use crate::core::HS;

mod core;

// These constants will be provided by the CLI later.
const DIR: &str = "./";

fn main() -> Result<(), io::Error> {
    let haystack = HS::new();

    haystack.run(DIR)
}