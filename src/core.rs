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
            return Result::Err(
                Error::new(ErrorKind::InvalidInput, "empty search term is not allowed")
            );
        }
        let mg = Manager {
            term: term.to_owned(),
            pool: vec![Worker{}; pool_size],
        };
        Result::Ok(mg)
    }

    fn take_file(&self, name: &str, buf: &[u8]) -> bool {
        let res = self.pool.last().unwrap().process(buf, &self.term);
        
        if res {
            println!("{}", name);
        }

        return res;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(self, mg: &Manager, dir: &str) -> Result<usize, io::Error> {
        let mut buf: Vec<u8> = Vec::new();
        let mut counter: usize = 0;

        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let mut handle = File::open(item.path())?;

                buf.clear();
                handle.read_to_end(&mut buf)?;

                if mg.take_file(item.path().to_str().unwrap(), &buf) {
                    counter = counter + 1;
                }
            }
        }
        Ok(counter)
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
            Err(err) => assert!(err.kind() == ErrorKind::InvalidInput && err.description() == "empty search term is not allowed",
                "wrong error returned: {}", err)
        }
    }

    #[test]
    fn empty_buffer() {
        let w = Worker {};
        let buf = vec![];
        assert!(!w.process(&buf, "text"),
            "empty buffer should return false");
    }

    #[test]
    fn find_at_end() {
        let w = Worker {};
        let buf = "0123456789".as_bytes();
        assert!(w.process(&buf, "789"),
            "finding the search term at the end should return true");
    }

    /// This test should NOT fail (e. g. index out of bounds)
    #[test]
    fn find_only_half_at_end() {
        let w = Worker {};
        let buf = "0123456789".as_bytes();
        assert!(!w.process(&buf, "8910"),
            "finding the pattern only half at the end should return false");
    }

    #[test]
    fn find_at_beginning() {
        let w = Worker {};
        let buf = "0123456789".as_bytes();
        assert!(w.process(&buf, "012"),
            "finding the pattern at the beginning should return true");
    }

    #[test]
    fn find_at_center() {
        let w = Worker {};
        let buf = "0123456789".as_bytes();
        assert!(w.process(&buf, "456"),
            "finding the pattern at the center should return true");
    }

    #[test]
    fn finding_nothing() {
        let w = Worker {};
        let buf = "0123456789".as_bytes();
        assert!(!w.process(&buf, "asdf"),
            "finding nothing should return false");
    }
}