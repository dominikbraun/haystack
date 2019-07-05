extern crate clap;

use clap::{App, Arg};

pub fn build() -> App<'static, 'static> {
    App::new("haystack")
        .version(env!("FULL_VERSION"))
        .arg(arg_dir())
        .arg(arg_needle())
        .arg(flag_snippets())
        .arg(flag_benchmark())
        .arg(arg_max_depth())
        .arg(arg_buf_size())
        .arg(arg_poolsize())
}

fn arg_dir() -> Arg<'static, 'static> {
    Arg::with_name("dir")
        .index(1)
        .help("The directory to be searched in.")
        .takes_value(true)
        .required(true)
}

fn arg_needle() -> Arg<'static, 'static> {
    Arg::with_name("needle")
        .index(2)
        .help("The text you want to search for.")
        .takes_value(true)
        .required(true)
}

fn flag_snippets() -> Arg<'static, 'static> {
    Arg::with_name("snippets")
        .short("s")
        .long("snippets")
        .help("Prints a text snippet containing the found search term.")
        .takes_value(false)
        .required(false)
}

fn flag_benchmark() -> Arg<'static, 'static> {
    Arg::with_name("benchmark")
        .long("benchmark")
        .help("Displays benchmarking data.")
        .takes_value(false)
        .required(false)
}

fn arg_max_depth() -> Arg<'static, 'static> {
    Arg::with_name("max_depth")
        .short("d")
        .long("max-depth")
        .help("Used buffer size for reading from the buffered reader.")
        .takes_value(true)
        .required(false)
}

fn arg_buf_size() -> Arg<'static, 'static> {
    Arg::with_name("buf_size")
        .long("bufsize")
        .help("Used buffer size for reading from the buffered reader.")
        .takes_value(true)
        .required(false)
}

fn arg_poolsize() -> Arg<'static, 'static> {
    Arg::with_name("poolsize")
        .long("poolsize")
        .help("The worker pool size, i. e. number of threads.")
        .takes_value(true)
        .required(false)
}