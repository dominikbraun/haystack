extern crate walkdir;
extern crate crossbeam;

use std::fs::File;
use std::path::Path;
use std::io;
use std::io::{Error, ErrorKind, Read};
use walkdir::WalkDir;
use crossbeam::channel as cc;

const CH_BUF_SIZE: u8 = 10;

#[derive(Debug, Clone)]
pub struct Manager {
    term: String,
    pool: Vec<Worker>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Result<Manager, Error> {
        if term.is_empty() {
            return Result::Err(Error::new(ErrorKind::InvalidInput, "empty search term is not allowed"));
        }
        let mg = Manager {
            term: term.to_owned(),
            pool: vec![Worker{}; pool_size],
        };
        Result::Ok(mg)
    }

    pub fn recv(&self, rx: cc::Receiver<String>) {
        let rx_iter: Vec<_> = rx.try_iter().collect();

        for job in rx_iter {
            println!("{:?}", job);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(&self, dir: &str, tx: cc::Sender<String>) -> Result<(), io::Error> {
        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let path = item.path().display().to_string();
                tx.send(path);
            }
        }
        drop(tx);
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct Worker {}

impl Worker {
    fn process(self, buf: &[u8], term: &str) -> bool {
        let term = term.as_bytes();

        'bytes: for (i, _) in buf.iter().enumerate() {
            if buf.len() - i < term.len() {
                return false;
            }
            for (j, term_b) in term.iter().enumerate() {
                if buf[i + j] != *term_b {
                    continue 'bytes;
                }
                if j == term.len() - 1 {
                    return true;
                }
            }
        }
        return false;
    }
}