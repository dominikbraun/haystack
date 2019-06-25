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
    work_tx: cc::Sender<String>,
    work_rx: cc::Receiver<String>,
    worker_finish_tx: cc::Sender<bool>,
    worker_finish_rx: cc::Receiver<bool>,
    counter: Arc<atomic_counter::RelaxedCounter>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Result<Manager, Error> {
        if term.is_empty() {
            return Result::Err(Error::new(ErrorKind::InvalidInput, "empty search term is not allowed"));
        }
        let (work_tx, work_rx) = cc::bounded(pool_size * 2);
        let (worker_finish_tx, worker_finish_rx) = cc::bounded(pool_size);
        let counter = Arc::new(atomic_counter::RelaxedCounter::new(0));

        let mg = Manager {
            term: term.to_owned(),
            pool_size,
            work_tx,
            work_rx,
            worker_finish_tx,
            worker_finish_rx,
            counter,
        };
        
        Result::Ok(mg)
    }
    
    pub fn start(&self, buf_size: usize) {
        for _ in 0..self.pool_size {
            let term = self.term.clone();
            let work_rx = self.work_rx.clone();
            let worker_finish_tx = self.worker_finish_tx.clone();
            let counter = self.counter.clone();

            thread::spawn(move || {
                Worker {
                    term,
                    buf_size,
                    counter,
                }.reicv(work_rx, worker_finish_tx);
            });
        }
    }

    pub fn recv(&self, job: String) {
        self.work_tx.send(job);
    }

    pub fn wait(&self) -> usize {
        loop {
            if self.worker_finish_rx.len() == self.pool_size {
                break;
            }
        }
        return self.counter.get();
    }
}

#[derive(Debug, Clone)]
pub struct Scanner {}

impl Scanner {
    pub fn run(&self, dir: String, mg: &Manager) -> Result<(), io::Error> {
        for item in WalkDir::new(dir).into_iter().filter_map(|i| i.ok()) {
            if item.file_type().is_file() {
                let path = item.path().display().to_string();
                mg.recv(path);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Worker {
    term: String,
    buf_size: usize,
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
        let mut buf = vec![0; self.buf_size];
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