extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::fs::{File, OpenOptions};
use std::io;
use std::time::Instant;

use clap::Error;
use clap::ErrorKind;
use slog::{debug, Drain, error, info, Level, Logger, o};

use crate::log::build_logger;

mod app;
mod core;
mod log;

fn main() {
    let log = build_logger();
    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap_or_else(|| {
        error_panic!(log, Error::with_description("'haystack' parameter needed", ErrorKind::ArgumentNotFound));
    });

    let term = matches.value_of("needle").unwrap_or_else(|| {
        error_panic!(log, Error::with_description("'needle' parameter needed", ErrorKind::ArgumentNotFound));
    });

    let buf_size = matches
        .value_of("bufsize")
        .unwrap_or("8192")
        .parse::<usize>()
        .unwrap_or_else(|err| {
            error_panic!(log, err);
        });

    let pool_size = matches
        .value_of("poolsize")
        .unwrap_or("8")
        .parse::<usize>()
        .unwrap_or_else(|err| {
            error_panic!(log, err);
        });
    
    let with_snippets = matches.is_present("snippets");
    
    let now = Instant::now();
    
    let total = run(log.new(o!("manager" => 1)), dir, term, pool_size, buf_size);

    if matches.is_present("benchmark") {
        println!("\nElapsed time:\n{} µs\n{} ms",
                 now.elapsed().as_micros(),
                 now.elapsed().as_millis());
    };

    match total {
        Ok(count) => println!("found {} times", count),
        Err(e) => {
            error_panic!(log, e);
        },
    };
}

fn run(log: Logger, dir: &str, term: &str, pool_size: usize, buf_size: usize) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(log, term, pool_size, buf_size);
    haystack.spawn();

    core::scan(dir, &haystack);

    Ok(haystack.stop())
}