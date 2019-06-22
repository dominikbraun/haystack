use std::io;

use crate::core::Manager;
use crate::core::Scanner;

mod core;
mod app;

fn main() -> Result<(), io::Error> {
    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap_or("./");
    let term = matches.value_of("needle").unwrap();

    let haystack = Manager::new(term, 5);
    let _ = Scanner{}.run(&haystack, dir);

    Ok(())
}