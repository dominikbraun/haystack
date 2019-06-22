extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::Read;

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Manager {
    term: String,
    pool: Vec<Worker>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Manager {
        Manager {
            term: term.to_owned(),
            pool: vec![Worker{}; pool_size],
        }
    }

    fn take_file(&self, buf: &Vec<u8>) {
        let res = self.pool.last().unwrap().process(buf, &self.term);
        println!("{:?}", res);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run<'a>(&self, mg: &'a Manager, dir: &str) -> Result<(), io::Error> {
        let mut buf: Vec<u8> = Vec::new();

        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let mut handle = File::open(item.path())?;

                buf.clear();
                handle.read_to_end(&mut buf)?;

                mg.take_file(&buf);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct Worker {}

impl Worker {
    fn process(&self, buf: &Vec<u8>, term: &str) -> bool {
        let term = term.as_bytes();

        'bytes: for (i, b) in buf.iter().enumerate() {
            for (j, term_b) in term.iter().enumerate() {
                if buf[i + j] != *term_b {
                    continue 'bytes;
                }
                if j == term.len() - 1 {
                    return true;
                }
            }
        }
        false
    }
}