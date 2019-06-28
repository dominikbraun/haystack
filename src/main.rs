extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::io;
use std::process::exit;
use std::time::Instant;

use clap::Error;
use clap::ErrorKind;
use slog::{Drain, error, info, Logger, o};

use crate::core::Manager;

mod core;
mod app;
mod exp;

const GIT_HASH: &str = env!("GIT_HASH");

macro_rules! error_panic {
    ($log:expr, $err:expr) => {
        error!(&$log, "{}", $err);
        panic!($err);
    };
}

fn main() -> Result<(), io::Error> {
    let log = logger();
    info!(log, "Starting Haystack");

    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack")
        .unwrap_or_else(|| { error_panic!(log, Error::with_description("'haystack' parameter needed", ErrorKind::ArgumentNotFound)); });

    let term = matches.value_of("needle")
        .unwrap_or_else(|| { error_panic!(log, Error::with_description("'needle' parameter needed", ErrorKind::ArgumentNotFound)); });

    let buf_size = matches
        .value_of("buffersize")
        .unwrap_or("5000")
        .parse::<usize>()
        .unwrap_or_else(|err| { error_panic!(log, err); });

    let pool_size = matches
        .value_of("poolsize")
        .unwrap_or("4")
        .parse::<usize>()
        .unwrap_or_else(|err| { error_panic!(log, err); });

    // start measuring execution time
    let now = Instant::now();

    let res = if matches.is_present("exp") {
        run_exp(dir, term, pool_size, buf_size)
    } else {
        run_stable(dir, term, pool_size, buf_size)
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

fn run_stable(dir: &str, term: &str, pool_size: usize, buf_size: usize) -> Result<usize, io::Error> {
    let haystack = Manager::new(term, pool_size)?;
    haystack.spawn(buf_size);

    core::scan(dir.to_owned(), &haystack);

    Ok(haystack.stop())
}

fn run_exp(dir: &str, term: &str, pool_size: usize, buf_size: usize) -> Result<usize, io::Error> {
    let haystack = exp::Manager::new(term, pool_size)?;

    haystack.start(buf_size);

    let dir = dir.to_owned();
    let _ = exp::Scanner{}.run(dir, &haystack);

    Ok(haystack.stop())
}

fn logger() -> Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    // ToDo: Fuse needed? (panics when Drain errors)
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    // Get a root logger that will log into a given drain.
    //
    // Note `o!` macro for more natural `OwnedKeyValue` sequence building.
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