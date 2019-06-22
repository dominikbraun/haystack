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
        if term.len() == 0 {
            return Result::Err(Error::new(ErrorKind::InvalidInput, "empty search term is not allowed"));
        }

        Result::Ok(Manager {
            term: term.to_owned(),
            pool: vec![Worker{}; pool_size],
        })
    }

    fn take_file(&self, name: &str, buf: &Vec<u8>) {
        let res = self.pool.last().unwrap().process(buf, &self.term);
        
        if res {
            println!("{}", name);
        }
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

                mg.take_file(item.path().to_str().unwrap(), &buf);
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

        'bytes: for (i, _) in buf.iter().enumerate() {
            if buf.len() - i <= term.len() {
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::ErrorKind;

    use crate::core::{Manager, Worker};

    #[test]
    fn empty_search_term() {
        let m = Manager::new("", 5);
        match m {
            Ok(_) => panic!("this call should return an error"),
            Err(err) => assert!(err.kind() == ErrorKind::InvalidInput && err.description() == "empty search term is not allowed", "wrong error returned: {}", err)
        }
    }

    #[test]
    fn empty_buffer() {
        let w = Worker {};
        let buf: Vec<u8> = vec![];
        assert!(!w.process(&buf, "text"), "empty buffer should return false");
    }
}