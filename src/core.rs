extern crate walkdir;

use walkdir::WalkDir;
use std::fs::File;
use std::io::Read;
use std::io;

#[derive(Debug)]
pub struct HS<'a, 'b> {
    sc: &'a Scanner,
    mg: &'b Manager,
}

#[derive(Debug, Copy, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(&self, path: &str) -> Result<(), io::Error> {
        let mut buf: Vec<u8> = Vec::new();

        for item in WalkDir::new(path).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let mut handle = File::open(item.path())?;

                buf.clear();
                handle.read_to_end(&mut buf)?;



                println!("opened {}", item.path().display().to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Manager {
    pool: Vec<Worker>,
}

#[derive(Debug, Copy, Clone)]
struct Worker {}