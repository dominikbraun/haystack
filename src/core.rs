extern crate atomic_counter;
extern crate crossbeam;
extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use atomic_counter::AtomicCounter;
use atomic_counter::RelaxedCounter;
use crossbeam::channel as cc;
use slog::{error, info, Logger, o};
use walkdir::WalkDir;

pub struct Manager {
    log: Logger,
    term: String,
    pool_size: usize,
    tx: cc::Sender<String>,
    rx: cc::Receiver<String>,
    done_tx: cc::Sender<bool>,
    done_rx: cc::Receiver<bool>,
    found: Arc<RelaxedCounter>,
}

impl Manager {
    pub fn new(log: Logger, term: &str, pool: usize) -> Result<Manager, Error> {
        info!(log, "setup manager");

        if term.is_empty() {
            return Result::Err(
                Error::new(ErrorKind::InvalidInput, "search term must not be empty")
            );
        }
        let (tx, rx) = cc::bounded(pool * 2);
        let (done_tx, done_rx) = cc::bounded(pool);
        let found = Arc::new(RelaxedCounter::new(0));

        let m = Manager {
            log,
            term: term.to_owned(),
            pool_size: pool,
            tx,
            rx,
            done_tx,
            done_rx,
            found,
        };

        Result::Ok(m)
    }

    pub fn spawn(&self, buf_size: usize) {
        for i in 0..self.pool_size {
            let term = self.term.clone();
            let rx = self.rx.clone();
            let done_tx = self.done_tx.clone();
            let found = self.found.clone();
            let log = self.log.new(o!("worker" => i));

            thread::spawn(move || {
                let w = Worker {
                    log,
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

    pub fn stop(&self) -> usize {
        // send empty string for each worker (empty string is command for closing)
        for _ in 0..self.pool_size {
            self.tx.send(String::new());
        }
        loop {
            if self.done_rx.len() == self.pool_size {
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
    log: Logger,
    term: String,
    buf_size: usize,
}

impl Worker {
    fn recv(&self, rx: cc::Receiver<String>, done_tx: cc::Sender<bool>, found: Arc<RelaxedCounter>) {
        info!(self.log, "starting worker");
        loop {
            if let Ok(file) = rx.recv() {
                // empty string is signal for closing worker
                if file.is_empty() {
                    break;
                }
                let handle = match File::open(Path::new(&file)) {
                    Ok(f) => f,
                    Err(err) => {
                        error!(self.log, "{}", err);
                        continue;
                    }
                };
                let mut reader = BufReader::new(handle);
                let count = self.process(&mut reader);

                if count > 0 {
                    info!(self.log, "found '{}' {} times in file {}", self.term, count, file);
                    found.add(count);
                }
            } else {
                break;
            }
        }
        info!(self.log, "stopping worker");
        done_tx.send(true).unwrap_or_else(|err| {
            error!(self.log, "{}", err);
        });
    }

    fn process(&self, reader: &mut Read) -> usize {
        let mut buf = vec![0; self.buf_size];
        let mut cursor = 0;
        let term = self.term.as_bytes();
        let mut counter = 0;

        loop {
            if let Ok(size) = reader.read(&mut buf) {
                if size == 0 {
                    break;
                }

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
                        counter = counter + 1;
                        cursor = 0;
                    }
                }
            } else {
                break;
            }
        }
        counter
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
    use std::sync::Arc;

    use crate::core::{Manager, Worker};

    fn setup_fake_file(data: &str) -> Cursor<Vec<u8>> {
        let mut fake_file = Cursor::new(Vec::new());

        // Write into the "file" and seek to the beginning
        fake_file.write_all(data.as_bytes()).unwrap();
        fake_file.seek(SeekFrom::Start(0)).unwrap();

        return fake_file
    }

    fn setup_test_worker(term: &str, buf_size: usize) -> Worker {
        return Worker {
            term: String::from(term),
            buf_size,
        };
    }

    #[test]
    fn empty_search_term() {
        let m = Manager::new("", 5);
        match m {
            Ok(_) => panic!("this call should return an error"),
            Err(err) => assert!(err.kind() == ErrorKind::InvalidInput && err.description() == "search term must not be empty",
                                "wrong error returned: {}", err)
        }
    }

    #[test]
    fn empty_buffer() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(!setup_test_worker("789", 0).process(&mut reader),
                "empty buffer should return false");
    }

    #[test]
    fn find_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(setup_test_worker("789", 5).process(&mut reader),
                "finding the search term at the end should return true");
    }

    /// This test should NOT fail (e. g. index out of bounds)
    #[test]
    fn find_only_half_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(!setup_test_worker("8910", 5).process(&mut reader),
                "finding the pattern only half at the end should return false");
    }

    #[test]
    fn find_at_beginning() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(setup_test_worker("012", 5).process(&mut reader),
                "finding the pattern at the beginning should return true");
    }

    #[test]
    fn find_at_center() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(setup_test_worker("34567", 5).process(&mut reader),
                "finding the pattern at the center should return true");
    }

    #[test]
    fn finding_nothing() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert!(!setup_test_worker("asdf", 5).process(&mut reader),
                "finding nothing should return false");
    }
}