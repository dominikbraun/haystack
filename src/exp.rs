extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};

use walkdir::WalkDir;

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

    fn take_file(&self, name: &str, buf: &[u8]) {
        let res = self.pool.last().unwrap().process(buf, &self.term);
        
        if res {
            println!("{}", name);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(self, mg: &Manager, dir: &str) -> Result<(), io::Error> {
        let mut buf: Vec<u8> = Vec::new();

        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let mut handle = File::open(item.path())?;

                buf.clear();
                handle.read_to_end(&mut buf)?;

                mg.take_file(item.path().to_str().unwrap(), &buf);
            }
        }
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