extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::fs::{OpenOptions};

use slog::{debug, Drain, error, info, Level, Logger, o};

const GIT_HASH: &str = env!("GIT_HASH");

#[macro_export(local_inner_macros)]
macro_rules! error_panic {
    ($log:expr, $err:expr) => {
        error!(&$log, "{}", $err);
        std::panic!($err);
    };
}

#[cfg(debug_assertions)]
pub fn build_logger() -> Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let version = build_version_str();

    return Logger::root(
        drain.fuse(),
        o!("version" => version),
    );
}


#[cfg(not(debug_assertions))]
pub fn build_logger() -> Logger {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("haystack.log")
        .unwrap();

    let decorator = slog_term::PlainDecorator::new(file);

    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let version = build_version_str();

    return Logger::root(
        drain.fuse(),
        o!("version" => version),
    );
}

fn build_version_str() -> String {
    let mut hash = String::new();

    if !GIT_HASH.is_empty() {
        hash.push_str("-");
        hash.push_str(GIT_HASH);
    }
    format!("v{}{}", env!("CARGO_PKG_VERSION"), hash)
}