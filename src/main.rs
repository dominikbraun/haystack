use std::io;
use std::thread;
use std::time::Instant;

use crossbeam::channel as cc;

use crate::core::Manager;
use crate::core::Scanner;

mod core;
mod app;
mod exp;

fn main() -> Result<(), io::Error> {
    let now = Instant::now();

    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap();
    let term = matches.value_of("needle").unwrap();

    let res = if matches.is_present("exp") {
        run_exp(dir, term)
    } else {
        run_stable(dir, term)
    };

    if matches.is_present("benchmark") {
        println!("\nElapsed time:\n{} µs", now.elapsed().as_micros());
        println!("{} ms", now.elapsed().as_millis());
        println!("{} s", now.elapsed().as_secs());
    }

    return res;
}

fn run_stable(dir: &str, term: &str) -> Result<(), io::Error> {
    let haystack = Manager::new(term, 5)?;
    let _ = Scanner{}.run(&haystack, dir);

    Ok(())
}

fn run_exp(dir: &str, term: &str) -> Result<(), io::Error> {
    let (tx, rx) = cc::unbounded();

    let haystack = exp::Manager::new(term, 5)?;

    let dir = dir.to_owned();
    thread::spawn(move || {
        let _ = exp::Scanner {}.run(dir, tx);
    });

    haystack.recv(rx);
    Ok(())
}