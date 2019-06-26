extern crate atomic_counter;
extern crate crossbeam;
extern crate walkdir;

use std::thread;
use std::sync::Arc;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek};
use std::path::Path;

use crossbeam::channel as cc;
use walkdir::WalkDir;
use atomic_counter::AtomicCounter;
use atomic_counter::RelaxedCounter;

pub struct Manager {
    term: String,
    pool: usize,
    tx: cc::Sender<String>,
    rx: cc::Receiver<String>,
    done_tx: cc::Sender<bool>,
    done_rx: cc::Receiver<bool>,
    found: Arc<RelaxedCounter>,
}

impl Manager {
    pub fn new(term: &str, pool: usize) -> Result<Manager, Error> {
        if term.is_empty() {
            return Result::Err(
                Error::new(ErrorKind::InvalidInput, "search term must not be empty")
            );
        }
        let (tx, rx) = cc::bounded(pool * 2);
        let (done_tx, done_rx) = cc::bounded(pool);
        let found = Arc::new(RelaxedCounter::new(0));

        let m = Manager {
            term: term.to_owned(),
            pool,
            tx,
            rx,
            done_tx,
            done_rx,
            found,
        };

        Result::Ok(m)
    }

    pub fn spawn(&self, buf_size: usize) {
        for _ in 0..self.pool {
            
            let term = self.term.clone();
            let rx = self.rx.clone();
            let done_tx = self.done_tx.clone();
            let found = self.found.clone();

            thread::spawn(move || {
                let w = Worker {
                    term,
                    buf_size
                };
                w.recv(rx, done_tx, found);
            });
        }
    }

    fn take(&self, file: String) {
        self.tx.send(file);
    }

    pub fn wait(&self) -> usize {
        for _ in 0..self.pool {
            self.tx.send(String::new());
        }
        loop {
            if self.done_rx.len() == self.pool {
                break;
            }
        }
        return self.found.get();
    }
}

pub fn scan(dir: String, mg: &Manager) -> Result<(), io::Error> {
    
    let items = WalkDir::new(dir).into_iter().filter_map(|i| {
        i.ok()
    });

    for i in items {
        if i.file_type().is_file() {
            let path = i.path().display().to_string();
            mg.take(path);
        }
    }
    Result::Ok(())
}

struct Worker {
    term: String,
    buf_size: usize,
}

impl Worker {
    fn recv(&self, rx: cc::Receiver<String>, done_tx: cc::Sender<bool>, found: Arc<RelaxedCounter>) {
        loop {
            if let Ok(file) = rx.recv() {
                if file.is_empty() {
                    break;
                }
                let mut handle = match File::open(Path::new(&file)) {
                    Ok(f) => f,
                    Err(e) => {
                        // ToDo: Log error
                        continue;
                    }
                };
                let mut reader = BufReader::new(handle);
                let was_found = self.process(&mut reader);

                if was_found {
                    // ToDo: Log success
                    found.inc();
                }
            } else {
                break;
            }
        }
        done_tx.send(true).unwrap_or_else(|v| {
            // ToDo: Log error
        });
    }

    fn process(&self, reader: &mut Read) -> bool {
        let mut buf = vec![0; self.buf_size];
        let mut cursor = 0;
        let term = self.term.as_bytes();

        loop {
            if let Ok(size) = reader.read(&mut buf) {
                for i in 0..size {
                    if buf[i] == term[cursor] {
                        cursor = cursor + 1;
                    } else if cursor > 0 {
                        if buf[i] == term[0] {
                            cursor = 1;
                        } else {
                            cursor = 0;
                        }
                    }

                    if cursor == term.len() {
                        return true;
                    }
                }
            } else {
                break;
            }
        }
        false
    }
}