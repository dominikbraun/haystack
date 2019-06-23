extern crate clap;

use clap::{App, Arg};

pub fn build() -> App<'static, 'static> {
    App::new("haystack")
        .arg(arg_dir())
        .arg(arg_needle())
        .arg(flag_recursive())
        .arg(flag_exp())
        .arg(flag_benchmark())
}

fn arg_dir() -> Arg<'static, 'static> {
    Arg::with_name("haystack")
        .short("h")
        .long("haystack")
        .help("The directory to be searched in.")
        .takes_value(true)
        .required(true)
}

fn arg_needle() -> Arg<'static, 'static> {
    Arg::with_name("needle")
        .short("n")
        .long("needle")
        .alias("term")
        .help("The text you want to search for.")
        .takes_value(true)
        .required(true)
}

fn flag_recursive() -> Arg<'static, 'static> {
    Arg::with_name("recursive")
        .short("r")
        .long("recursive")
        .help("Search files in all subdirectories, too.")
        .takes_value(false)
        .required(false)
}

fn flag_exp() -> Arg<'static, 'static> {
    Arg::with_name("exp")
        .short("e")
        .long("exp")
        .help("Use experimental, non-stable techniques")
        .takes_value(false)
        .required(false)
}

fn flag_benchmark() -> Arg<'static, 'static> {
    Arg::with_name("benchmark")
        .short("b")
        .long("benchmark")
        .help("Benchmark the search")
        .takes_value(false)
        .required(false)
}