extern crate atomic_counter;
extern crate crossbeam;
extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use crossbeam::channel as cc;
use walkdir::WalkDir;

use self::atomic_counter::AtomicCounter;

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

    pub fn recv(&self, rx: cc::Receiver<String>, trim_size: usize) -> usize {
        let (worker_finish_tx, worker_finish_rx) = cc::bounded(self.pool_size);
        let counter = Arc::new(atomic_counter::RelaxedCounter::new(0));

        // worker_finish_ has to live longer than work_ else --> deadlock
        {
            let (work_tx, work_rx) = cc::bounded(self.pool_size * 2);

            for _ in 0..self.pool_size {
                let term = self.term.clone();
                let work_rx = work_rx.clone();
                let worker_finish_tx = worker_finish_tx.clone();
                let counter = counter.clone();

                thread::spawn(move || {
                    Worker {
                        term,
                        trim_size,
                        counter,
                    }.reicv(work_rx, worker_finish_tx);
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

        return counter.get();
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
    trim_size: usize,
    counter: Arc<atomic_counter::RelaxedCounter>,
}

impl Worker {
    fn reicv(&self, work_rx: cc::Receiver<String>, finished: cc::Sender<bool>) {
        loop {
            match work_rx.try_recv() {
                Ok(job) => {
                    let mut handle = match File::open(Path::new(&job)) {
                        Ok(h) => h,
                        Err(err) => {
                            println!("Error while reading file {}: {}", job, err);
                            continue;
                        }
                    };

                    let mut reader = BufReader::new(handle);

                    let positive = self.process(&mut reader, &self.term);

                    if positive {
                        println!("Found in file {}", job);
                        self.counter.inc();
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

    fn process(&self, reader: &mut BufReader<File>, term: &str) -> bool {
        let mut buf = vec![0; self.trim_size];
        let mut term_cursor = 0;
        let term = term.as_bytes();

        loop {
            match reader.read(&mut buf) {
                Ok(size) => {
                    if size == 0 {
                        return false;
                    }

                    for i in 0..size {
                        if buf[i] == term[term_cursor] {
                            term_cursor = term_cursor + 1;
                        } else if term_cursor > 0 {
                            if buf[i] == term[0] {
                                term_cursor = 1;
                            } else {
                                term_cursor = 0;
                            }
                        }

                        if term_cursor == term.len() {
                            return true;
                        }
                    }
                }
                Err(err) => {
                    return false;
                }
            }
        }

        true

        /*

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
                return false;*/
    }
}