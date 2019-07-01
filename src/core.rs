extern crate crossbeam;
extern crate walkdir;

use std::fs;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;
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
    gate: sync::WaitGroup,
    total: Arc<AtomicU16>,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Manager {
        Manager {
            term: term.to_owned(),
            queue: Arc::new(Injector::<String>::new()),
            pool_size,
            gate: sync::WaitGroup::new(),
            total: Arc::new(AtomicU16::new(0)),
        }
    }

    pub fn spawn(&self) -> bool {
        for _ in 0..self.pool_size {
            let term = self.term.clone();
            let queue = Arc::clone(&self.queue);
            let gate = self.gate.clone();
            let total = Arc::clone(&self.total);

            thread::spawn(move || {
                loop {
                    if let Steal::Success(f) = queue.steal() {
                        if f.is_empty() {
                            break;
                        }
                        let path = Path::new(&f);
                        
                        let handle = match fs::File::open(path) {
                            Ok(h) => h,
                            Err(_) => { continue; },
                        };

                        if process(&term, handle) > 0 {
                            let mut val = total.load(Ordering::Relaxed);
                            total.store(val + 1, Ordering::Relaxed);
                        }
                    }
                }
                drop(gate);
            });
        }
        true
    }

    fn take(&self, file: String) {
        self.queue.push(file);
    }

    pub fn stop(self) -> u16 {
        for _ in 0..self.pool_size {
            self.queue.push(String::new());
        }
        self.gate.wait();
        
        return self.total.load(Ordering::Relaxed);
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

fn process(term: &str, handle: fs::File) -> u16 {
    let mut reader = BufReader::new(handle);
    let mut buf = vec![0; 8000];

    let mut cursor = 0;
    let mut found: u16 = 0;

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