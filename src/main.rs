extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::fs::{File, OpenOptions};
use std::io;
use std::time::Instant;

use clap::Error;
use clap::ErrorKind;
use slog::{debug, Drain, error, info, Level, Logger, o};

mod core;
mod app;

const GIT_HASH: &str = env!("GIT_HASH");

macro_rules! error_panic {
    ($log:expr, $err:expr) => {
        error!(&$log, "{}", $err);
        panic!($err);
    };
}

fn main() -> Result<(), io::Error> {
    let log = logger();
    debug!(log, "Starting Haystack");

    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap_or_else(|| {
        error_panic!(log, Error::with_description("'haystack' parameter needed", ErrorKind::ArgumentNotFound));
    });

    let term = matches.value_of("needle").unwrap_or_else(|| {
        error_panic!(log, Error::with_description("'needle' parameter needed", ErrorKind::ArgumentNotFound));
    });

    let buf_size = matches
        .value_of("buffersize")
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

    // start measuring execution time
    let now = Instant::now();

    let res = run(dir, term, pool_size, buf_size);

    if matches.is_present("benchmark") {
        println!("\nElapsed time:\n{} µs\n{} ms\n{} s",
                 now.elapsed().as_micros(),
                 now.elapsed().as_millis(),
                 now.elapsed().as_secs());
    };

    match res {
        Ok(count) => println!("found {} times", count),
        Err(err) => {
            error!(log, "{}", err);
            return Err(err)
        },
    };

    return Ok(());
}

fn run(dir: &str, term: &str, pool_size: usize, buf_size: usize) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(term, pool_size, buf_size);
    haystack.spawn();

    core::scan(dir, &haystack);

    Ok(haystack.stop() as u32)
}

#[cfg(debug_assertions)]
fn logger() -> Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    // ToDo: Fuse needed? (panics when Drain errors)
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let git_hash: String = if !GIT_HASH.is_empty() {
        format!("-{}", GIT_HASH)
    } else {
        String::new()
    };

    return Logger::root(
        drain.fuse(),
        o!("version" => format!("v{}{}", env!("CARGO_PKG_VERSION"), git_hash)),
    );
}


#[cfg(not(debug_assertions))]
fn logger() -> Logger {
    let log_path = "haystack.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let decorator = slog_term::PlainDecorator::new(file);
    // ToDo: Fuse needed? (panics when Drain errors)
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let git_hash: String = if !GIT_HASH.is_empty() {
        format!("-{}", GIT_HASH)
    } else {
        String::new()
    };

    return Logger::root(
        drain.fuse(),
        o!("version" => format!("v{}{}", env!("CARGO_PKG_VERSION"), git_hash)),
    );
}