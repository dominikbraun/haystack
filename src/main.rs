use std::io;
use std::borrow::Cow;
use std::time::Instant;

mod app;
mod core;

pub struct Settings {
    _snippets: bool,
    case_insensitive: bool,
    benchmark: bool,
    max_depth: Option<usize>,
    buf_size: usize,
    pool_size: usize,
}

fn main() {
    let m = app::build().get_matches();

    let dir = m.value_of("dir").unwrap();
    let needle = m.value_of("needle").unwrap();

    let case_insensitive = m.is_present("case_insensitive");

    let needle: Cow<str> = if case_insensitive {
        needle.to_ascii_lowercase().into()
    } else {
        needle.into()
    };

    let snippets = m.is_present("snippets");
    let benchmark = m.is_present("benchmark");
    
    let max_depth = m.value_of("max_depth").map(|d| {
        d.parse::<usize>().unwrap()
    });

    let buf_size = m.value_of("buf_size")
        .unwrap_or("8192")
        .parse::<usize>()
        .unwrap();

    let pool_size = m.value_of("poolsize")
        .unwrap_or("8")
        .parse::<usize>()
        .unwrap();

    let options = Settings {
        _snippets: snippets,
        case_insensitive,
        benchmark,
        max_depth,
        buf_size,
        pool_size,
    };
    
    let now = Instant::now();

    let total = run(dir, &needle, &options);

    if options.benchmark {
        println!("\nElapsed time: {} ms", now.elapsed().as_millis());
    }

    match total {
        Ok(count) => println!("found {} times", count),
        Err(e) => {
            eprintln!("{}", e);
        },
    };
}

fn run(dir: &str, term: &str, options: &Settings) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(term, options);
    haystack.spawn();

    core::scan(dir, &haystack).map(|_| haystack.stop())
}