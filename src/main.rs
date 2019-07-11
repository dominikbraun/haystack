extern crate structopt;

use std::io;
use std::path::PathBuf;
use std::time::Instant;

use structopt::StructOpt;

mod core;

#[derive(Debug, StructOpt)]
pub struct Params {
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
    let mut opt = Params::from_args();

    if opt.case_insensitive {
        opt.needle = opt.needle.to_ascii_lowercase();
    }
    let now = Instant::now();

    let total = run(&opt);

    if opt.benchmark {
        println!("\nElapsed time: {} ms", now.elapsed().as_millis());
    }
    println!("{} occurences", total.unwrap());
}

fn run(args: &Params) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(args);
    haystack.spawn();

    core::scan(&args.dir, &haystack)
        .map(|_| haystack.stop())
}