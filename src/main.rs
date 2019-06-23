use std::io;
use std::thread;

use crossbeam::channel as cc;

use crate::core::Manager;
use crate::core::Scanner;

mod core;
mod app;
mod exp;

fn main() -> Result<(), io::Error> {
    let matches = app::build().get_matches();

    let dir = matches.value_of("haystack").unwrap();
    let term = matches.value_of("needle").unwrap();

    if matches.is_present("exp") {
        run_exp(dir, term)
    } else {
        run_stable(dir, term)
    }
}

fn run_stable(dir: &str, term: &str) -> Result<(), io::Error> {
    let haystack = Manager::new(term, 5)?;
    let _ = Scanner{}.run(&haystack, dir);

    Ok(())
}

fn run_exp(dir: &str, term: &str) -> Result<(), io::Error> {
    let (tx, rx) = cc::unbounded();

    let haystack = exp::Manager::new(term, 5)?;
    let haystack2 = exp::Manager::new(term, 5)?;
    let haystack3 = exp::Manager::new(term, 5)?;
    let haystack4 = exp::Manager::new(term, 5)?;
    let haystack5 = exp::Manager::new(term, 5)?;

    let rx2 = rx.clone();
    let rx3 = rx.clone();
    let rx4 = rx.clone();
    let rx5 = rx.clone();


    let dir = dir.to_owned();
    thread::spawn(move || {
        let _ = exp::Scanner {}.run(dir, tx);
    });

    thread::spawn(move || {
        let _ = haystack.recv(rx, "1");
    });

    thread::spawn(move || {
        let _ = haystack2.recv(rx2, "2");
    });

    thread::spawn(move || {
        let _ = haystack3.recv(rx3, "3");
    });

    thread::spawn(move || {
        let _ = haystack4.recv(rx4, "4");
    });


    let _ = haystack5.recv(rx5, "5");

    Ok(())
}