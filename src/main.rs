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
    let trim_size = matches
        .value_of("trimsize")
        .unwrap_or("500000000")
        .parse::<usize>()
        .unwrap();

    let pool_size = matches
        .value_of("ps")
        .unwrap_or("4")
        .parse::<usize>()
        .unwrap();

    let res = if matches.is_present("exp") {
        run_exp(dir, term, pool_size, trim_size)
    } else {
        run_stable(dir, term)
    };

    if matches.is_present("benchmark") {
        println!("\nElapsed time:\n{} µs", now.elapsed().as_micros());
        println!("{} ms", now.elapsed().as_millis());
        println!("{} s", now.elapsed().as_secs());
    };

    match res {
        Ok(count) => println!("found {} times", res.unwrap()),
        Err(err) => return Err(err),
    };

    return Ok(());
}

fn run_stable(dir: &str, term: &str) -> Result<usize, io::Error> {
    let haystack = Manager::new(term, 5)?;
    let res = Scanner {}.run(&haystack, dir);

    return res;
}

fn run_exp(dir: &str, term: &str, pool_size: usize, trim_size: usize) -> Result<usize, io::Error> {
    let haystack = exp::Manager::new(term, pool_size)?;

    haystack.start(trim_size);

    let dir = dir.to_owned();
    let _ = exp::Scanner{}.run(dir, &haystack);

    Ok(haystack.stop())
}