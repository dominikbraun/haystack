extern crate structopt;

use std::io;
use std::path::PathBuf;
use std::time::Instant;

use structopt::StructOpt;

mod core;

// Defines the CLI interface and creates a clap app at compile time.
// Params guarantees that all fields have been initialized with an
// appropriate value, i. e. they can be accessed without any checks.
#[derive(Debug, StructOpt)]
pub struct Params {
    /// The directory to be searched in
    #[structopt(parse(from_os_str))]
    dir: PathBuf,

    /// The text you want to search for
    needle: String,

    /// Prints a text snippet showing the result
    #[structopt(short, long)]
    snippets: bool,

    /// extension whitelist as comma separated list
    #[structopt(short, long)]
    whitelist: Vec<String>,

    /// Use case-insensivie search (may be slower)
    #[structopt(name="case-insensitive", short, long)]
    case_insensitive: bool,

    /// Measure the execution time
    #[structopt(short, long)]
    benchmark: bool,

    /// Defines the depth of recursion
    #[structopt(short="d", long="max-depth")]
    max_depth: Option<usize>,

    /// The size of the buffer used for the file
    #[structopt(long="bufsize", default_value="8192")]
    buf_size: usize,

    /// The number of worker threads
    #[structopt(long="poolsize", default_value="8")]
    pool_size: usize
}

fn main() {
    let mut opt = Params::from_args();

    // If the --case-insensitive flag has been set, the search
    // term will be lowercased for comparison.
    if opt.case_insensitive {
        opt.needle = opt.needle.to_ascii_lowercase();
    }
    let now = Instant::now();

    // Run haystack and get the total number of files found.
    let total = run(&opt);

    if opt.benchmark {
        println!("\nElapsed time: {} ms", now.elapsed().as_millis());
    }
    println!("{} occurences", total.unwrap());
}

// Creates a new haystack Manager using the parsed CLI args, spawns
// all worker threads and starts scanning the directories. When all
// files were processed, the workers will be instructed to stop.
fn run(args: &Params) -> Result<u32, io::Error> {
    let haystack = core::Manager::new(args);
    haystack.spawn();

    core::scan(&args.dir, &haystack)
        .map(|_| haystack.stop())
}
