extern crate crossbeam;
extern crate walkdir;

use std::fs;
use std::io;
use std::io::{BufReader, Error, Read};
use std::io::{BufWriter, Stdout, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;

use crossbeam::deque::Injector;
use crossbeam::deque::Steal;
use crossbeam::sync;
use walkdir::WalkDir;

pub struct Manager {
    term: String,
    queue: Arc<Injector<String>>,
    pool_size: usize,
    buf_size: usize,
    done_tx: crossbeam::Sender<u32>,
    done_rx: crossbeam::Receiver<u32>,
    total: Arc<AtomicU16>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize, buf_size: usize) -> Manager {
        let stdout = BufWriter::new(io::stdout());
        let (done_tx, done_rx) = crossbeam::bounded(pool_size);

        Manager {
            term: term.to_owned(),
            queue: Arc::new(Injector::<String>::new()),
            pool_size,
            buf_size,
            done_tx,
            done_rx,
            total: Arc::new(AtomicU16::new(0)),
        }
    }

    pub fn spawn(&self) -> bool {
        for _ in 0..self.pool_size {
            let term = self.term.clone();
            let queue = Arc::clone(&self.queue);
            let total = Arc::clone(&self.total);
            let mut stdout = BufWriter::new(io::stdout());
            let buf_size = self.buf_size.clone();
            let done_tx = self.done_tx.clone();

            thread::spawn(move || {
                let mut found: u32 = 0;
                loop {
                    if let Steal::Success(f) = queue.steal() {
                        if f.is_empty() {
                            // empty string is the signal for closing
                            break;
                        }
                        let path = Path::new(&f);

                        let mut handle = match fs::File::open(path) {
                            Ok(h) => h,
                            Err(e) => { continue; },
                        };

                        let val = process(&term, &mut handle, buf_size);

                        if val > 0 {
                            found = found + val as u32;

                            stdout.write_all(format!("found {} times in {}\n", val, &f).as_bytes());
                        }
                    }
                }

                done_tx.send(found).unwrap_or_else(|err| {
                    println!("{}", err);
                });
            });
        }
        true
    }

    fn take(&self, file: String) {
        self.queue.push(file);
    }

    pub fn stop(&self) -> u32 {
        // send empty string for each worker (empty string is command for closing)
        for _ in 0..self.pool_size {
            self.queue.push(String::new());
        }

        loop {
            if self.done_rx.len() == self.pool_size {
                break;
            }
        }

        let mut sum: u32 = 0;
        for _ in 0..self.pool_size {
            sum = sum + self.done_rx.recv().unwrap_or(0);
        }

        return sum;
    }
}

pub fn scan(dir: &str, manager: &Manager) -> Result<(), io::Error> {
    
    let items = WalkDir::new(dir.to_owned()).into_iter().filter_map(|i| {
        i.ok()
    });

    for i in items {
        if i.file_type().is_file() {
            let path = i.path().display().to_string();
            manager.take(path);
        }
    }
    Result::Ok(())
}

fn process(term: &str, handle: &mut dyn Read, buf_size: usize) -> u32 {
    let mut reader = BufReader::new(handle);
    let mut buf = vec![0; buf_size];

    let mut cursor = 0;
    let mut found: u32 = 0;

    let term = term.as_bytes();

    loop {
        if let Ok(len) = reader.read(&mut buf) {
            if len == 0 {
                break;
            }
            
            for i in 0..len {
                if buf[i] == term[cursor] {
                    cursor += 1;
                } else if cursor > 0 {
                    if buf[i] == term[0] {
                        cursor = 1;
                    } else {
                        cursor = 0;
                    }
                }

                if cursor == term.len() {
                    found += 1;
                    cursor = 0;
                }
            }
        } else {
            return 0;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
    use std::sync::Arc;

    use slog::{Drain, Logger, o};

    use crate::exp::{Manager, process};

    fn setup_fake_file(data: &str) -> Cursor<Vec<u8>> {
        let mut fake_file = Cursor::new(Vec::new());

        // Write into the "file" and seek to the beginning
        fake_file.write_all(data.as_bytes()).unwrap();
        fake_file.seek(SeekFrom::Start(0)).unwrap();

        return fake_file
    }

    fn logger() -> Logger {
        let decorator = slog_term::PlainDecorator::new(std::io::stdout());
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        return Logger::root(
            drain,
            o!(),
        );
    }

    #[test]
    fn find_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(1, process("789", &mut reader, 5), "finding the search term at the end should return true");
    }

    /// This test should NOT fail (e. g. index out of bounds)
    #[test]
    fn find_only_half_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(0, process("8910", &mut reader, 5), "finding the pattern only half at the end should return false");
    }

    #[test]
    fn find_at_beginning() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(1, process("012", &mut reader, 5), "finding the pattern at the beginning should return true");
    }

    #[test]
    fn find_at_center() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(1, process("34567", &mut reader, 5), "finding the pattern at the center should return true");
    }

    #[test]
    fn finding_nothing() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(0, process("asdf", &mut reader, 5), "finding nothing should return false");
    }

    #[test]
    fn find_several_times() {
        let mut reader = BufReader::new(setup_fake_file("abc01234abc56789abcjab"));
        assert_eq!(3, process("abc", &mut reader, 10), "the pattern should exist 3 times in the file");
    }
}