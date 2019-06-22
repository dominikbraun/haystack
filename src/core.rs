extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::Read;

use walkdir::WalkDir;

#[derive(Debug)]
pub struct HS {
    sc: Scanner,
    mg: Manager,
}

impl HS {
    pub fn new() -> HS {
        HS {
            sc: Scanner {},
            mg: Manager { pool: vec![] },
        }
    }

    pub fn search(&self, path: &str, term: &str) -> Result<(), io::Error> {
        self.sc.run(path, term)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(&self, path: &str, term: &str) -> Result<(), io::Error> {
        let mut buf: Vec<u8> = Vec::new();

        for item in WalkDir::new(path).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let mut handle = File::open(item.path())?;

                buf.clear();
                handle.read_to_end(&mut buf)?;

                let result = self.processFile(&buf, term);

                println!("opened {}: {}", item.path().display().to_string(), result);
            }
        }
        Ok(())
    }

    fn processFile(&self, fileBuf: &Vec<u8>, term: &str) -> bool {
        let term = term.as_bytes();

        'fileLoop: for (i, byte) in fileBuf.iter().enumerate() {
            for (j, searchByte) in term.iter().enumerate() {
                if fileBuf[i + j] != *searchByte {
                    continue 'fileLoop;
                }
                println!("------------------{}", String::from_utf8(vec!(*byte)).unwrap());
                if j == term.len() - 1 {
                    return true;
                }
            }
        };

        false
    }
}

#[derive(Debug)]
struct Manager {
    pool: Vec<Worker>,
}

#[derive(Debug, Copy, Clone)]
struct Worker {}