#[macro_use]
extern crate structopt;

use std::io;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Instant;

use structopt::StructOpt;

mod app;
mod core;

#[derive(Debug, StructOpt)]
#[structopt(name = "haystack")]
pub struct Settings {
    /// The directory to be searched in.
    #[structopt(parse(from_os_str))]
    dir: PathBuf,

    /// The text you want to search for.
    needle: String,

    /// Prints a text snippet containing the found search term.
    #[structopt(short, long)]
    snippets: bool,

    /// Enables case INsensitive search. Be careful, this may be slower.
    #[structopt(name="case-insensitive", short, long)]
    case_insensitive: bool,

    /// Displays benchmarking data.
    #[structopt(short, long)]
    benchmark: bool,

    /// Used buffer size for reading from the buffered reader.
    #[structopt(short="d", long="max-depth")]
    max_depth: Option<usize>,

    /// Used buffer size for reading from the buffered reader.
    #[structopt(long="bufsize", default_value="8192")]
    buf_size: usize,

    /// The worker pool size, i. e. number of threads.
    #[structopt(long="poolsize", default_value="8")]
    pool_size: usize
}

fn main() {
    let m = app::build().get_matches();

    let dir = m.value_of("dir").unwrap();
    let needle = m.value_of("needle").unwrap();

    // let case_insensitive = m.is_present("case_insensitive");

    // let snippets = m.is_present("snippets");
    // let benchmark = m.is_present("benchmark");
    
    // let max_depth = m.value_of("max_depth").map(|d| {
    //     d.parse::<usize>().unwrap()
    // });

    // let buf_size = m.value_of("buf_size")
    //     .unwrap_or("8192")
    //     .parse::<usize>()
    //     .unwrap();

    // let pool_size = m.value_of("poolsize")
    //     .unwrap_or("8")
    //     .parse::<usize>()
    //     .unwrap();

    // let options = Settings {
    //     _snippets: snippets,
    //     case_insensitive,
    //     benchmark,
    //     max_depth,
    //     buf_size,
    //     pool_size,
    // };

    let opt = Settings::from_args();

    let needle: Cow<str> = if opt.case_insensitive {
        needle.to_ascii_lowercase().into()
    } else {
        needle.into()
    };
    
    let now = Instant::now();

    let total = run(dir, &needle, &opt);

    if opt.benchmark {
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