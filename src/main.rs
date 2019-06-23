use std::io;

use crate::core::Manager;
use crate::core::Scanner;

mod core;
mod app;
mod exp;

fn main() -> Result<(), io::Error> {
    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap();
    let term = matches.value_of("needle").unwrap();

    if matches.is_present("exp") {
        run_exp(dir, term)
    } else {
        run_stable(dir, term)
    }
}

fn run_stable(dir: &str, term: &str) -> Result<(), io::Error> {
    let haystack = Manager::new(term, 5)?;
    let _ = Scanner{}.run(&haystack, dir);

    Ok(())
}

fn run_exp(dir: &str, term: &str) -> Result<(), io::Error> {
    let haystack = exp::Manager::new(term, 5)?;
    let _ = exp::Scanner{}.run(&haystack, dir);

    Ok(())
}