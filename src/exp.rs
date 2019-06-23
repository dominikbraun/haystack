extern crate crossbeam;
extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use std::thread;

use crossbeam::channel as cc;
use walkdir::WalkDir;

const WORKER_BUF_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct Manager {
    term: String,
    pool_size: usize,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Result<Manager, Error> {
        if term.is_empty() {
            return Result::Err(Error::new(ErrorKind::InvalidInput, "empty search term is not allowed"));
        }
        let mg = Manager {
            term: term.to_owned(),
            pool_size,
        };
        Result::Ok(mg)
    }

    pub fn recv(&self, rx: cc::Receiver<String>) {
        let (worker_finish_tx, worker_finish_rx) = cc::bounded(self.pool_size);

        // worker_finish_ has to live longer than work_ else --> deadlock
        {
            let (work_tx, work_rx) = cc::bounded(WORKER_BUF_SIZE);

            for _ in 0..self.pool_size {
                let term = self.term.clone();
                let work_rx = work_rx.clone();
                let worker_finish_tx = worker_finish_tx.clone();

                thread::spawn(move || {
                    Worker { term }.reicv(work_rx, worker_finish_tx);
                });
            }

            loop {
                match rx.try_recv() {
                    Ok(job) => {
                        work_tx.send(job);
                    },
                    Err(e) => {
                        if e.is_disconnected() {
                            break;
                        }
                    },
                }
            }
        }

        loop {
            if worker_finish_rx.len() == self.pool_size { // wait  until all workers are done
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(&self, dir: String, tx: cc::Sender<String>) -> Result<(), io::Error> {
        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let path = item.path().display().to_string();
                tx.send(path);
            }
        }
        loop {
            if tx.is_empty() {
                break;
            }
        }
        drop(tx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Worker {
    term: String,
}

impl Worker {
    fn reicv(&self, work_rx: cc::Receiver<String>, finished: cc::Sender<bool>) {
        let mut buf = Vec::new();

        loop {
            match work_rx.try_recv() {
                Ok(job) => {
                    let mut handle = File::open(Path::new(&job)).unwrap();
                    buf.clear();
                    handle.read_to_end(&mut buf);

                    let positive = self.process(&buf, &self.term);

                    if positive {
                        println!("Found in file {}", job);
                    }
                },
                Err(e) => {
                    if e.is_disconnected() {
                        break;
                    }
                },
            }
        }
        finished.send(true).unwrap();
    }

    fn process(&self, buf: &[u8], term: &str) -> bool {
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