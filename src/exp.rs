extern crate crossbeam;
extern crate walkdir;

use std::fs::File;
use std::io;
use std::io::{BufReader, Error, Read};
use std::iter;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use crossbeam::sync::WaitGroup;
use walkdir::WalkDir;

pub struct Manager {
    queue: Arc<Injector<String>>,
    term: String,
    pool_size: usize,
    wg: WaitGroup,
    total: Arc<AtomicU16>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Manager {
        Manager {
            queue: Arc::new(Injector::<String>::new()),
            term: term.to_owned(),
            pool_size,
            wg: WaitGroup::new(),
            total: Arc::new(AtomicU16::new(0)),
        }
    }

    pub fn spawn(&self) {
        for i in 0..self.pool_size {
            let term = self.term.clone();
            let wg = self.wg.clone();
            let total = Arc::clone(&self.total);
            let q = Arc::clone(&self.queue);
            
            thread::spawn(move || {
                loop {
                    if let Steal::Success(f) = q.steal() {
                        if f.is_empty() { break; }

                        let handle = match File::open(Path::new(&f)) {
                            Ok(f) => f,
                            Err(err) => {
                                continue;
                            }
                        };
                        let occurences = process(&term, handle);

                        if occurences > 0 {
                            let mut val = total.load(Ordering::Relaxed);
                            val += 1;
                            total.store(val, Ordering::Relaxed);
                            // LOG SUCCESS
                        }
                    }
                }
                drop(wg);
            });
        }
    }

    fn take(&self, file: String) {
        self.queue.push(file);
    }
    
    pub fn stop(self) -> u16 {
        for _ in 0..self.pool_size {
            self.queue.push(String::new());
        }
        self.wg.wait();
        
        return self.total.load(Ordering::Relaxed);
    }
}

pub fn scan(dir: &str, mg: &Manager) -> Result<(), io::Error> {
    
    let items = WalkDir::new(dir.to_owned()).into_iter().filter_map(|i| {
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

fn process(term: &str, handle: File) -> usize {
    let mut reader = BufReader::new(handle);
    let mut buf = vec![0; 8000];

    let mut cursor = 0;
    let mut positives = 0;

    let term = term.as_bytes();

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
                    positives += 1;
                    cursor = 0;
                }
            }
        } else {
            break;
        }
    }
    positives
}