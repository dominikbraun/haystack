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

    pub fn stop(&self) -> usize {
        // send empty string for each worker (empty string is command for closing)
        for i in 0..self.pool_size {
            self.work_tx.send(String::new());
        }

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
        };
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
            match work_rx.recv() {
                Ok(job) => {
                    // empty string is signal for closing worker
                    if job.is_empty() {
                        break;
                    }

                    let mut handle = match File::open(Path::new(&job)) {
                        Ok(h) => h,
                        Err(err) => {
                            println!("Error while reading file {}: {}", job, err);
                            continue;
                        }
                    };

                    let mut reader = BufReader::new(handle);

                    let positive = self.process(&mut reader);

                    if positive {
                        println!("Found in file {}", job);
                        self.counter.inc();
                    }
                },
                Err(e) => {
                    break;
                },
            }
        };
        finished.send(true).unwrap();
    }

    fn process(&self, reader: &mut Read) -> bool {
        let mut buf = vec![0; self.buf_size];
        let mut term_cursor = 0;
        let term = self.term.as_bytes();

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
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
    use std::sync::Arc;

    use crate::exp::{Manager, Worker};

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
            counter: Arc::default(),
            buf_size,
        };
    }

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