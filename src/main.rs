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

pub struct Settings {
    snippets: bool,
    benchmark: bool,
    max_depth: Option<usize>,
    buf_size: usize,
    pool_size: usize,
}

fn main() {
    let m = app::build().get_matches();
    let log = build_logger();

    let dir = m.value_of("dir").unwrap();
    let needle = m.value_of("needle").unwrap();

    let snippets = m.is_present("snippets");
    let benchmark = m.is_present("benchmark");
    
    let max_depth = m.value_of("max_depth").map(|d| {
        d.parse::<usize>().unwrap_or_else(|err| {
            error_panic!(log, err);
        })
    });

    let buf_size = m.value_of("buf_size")
        .unwrap_or("8192")
        .parse::<usize>()
        .unwrap_or_else(|err| {
            error_panic!(log, err);
        });

    let pool_size = m.value_of("poolsize")
        .unwrap_or("8")
        .parse::<usize>()
        .unwrap_or_else(|err| {
            error_panic!(log, err);
        });

    let options = Settings {
        snippets, benchmark, max_depth, buf_size, pool_size,
    };
    
    let now = Instant::now();

    let total = run(log.new(o!("manager" => 1)), dir, needle, &options);

    if options.benchmark {
        println!("\nElapsed time: {} ms", now.elapsed().as_millis());
    }

    match total {
        Ok(count) => println!("found {} times", count),
        Err(e) => {
            error_panic!(log, e);
        },
    };
}

fn run(log: Logger, dir: &str, term: &str, options: &Settings) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(log, term, options);
    haystack.spawn();

    core::scan(dir, &haystack);

    Ok(haystack.stop())
}