use std::io;

use crate::core::Manager;
use crate::core::Scanner;

mod core;

// These constants will be provided by the CLI later.
const DIR: &str = "./";
const TERM: &str = "package";

fn main() -> Result<(), io::Error> {
    let haystack = Manager::new(TERM, 5);
    let _ = Scanner{}.run(&haystack, DIR);

    Ok(())
}