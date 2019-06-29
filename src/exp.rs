use std::io;
use std::iter;
use std::thread;

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use crossbeam::sync::WaitGroup;
use walkdir::WalkDir;

pub struct Manager {
    queue: Worker<String>,
    term: String,
    pool_size: usize,
    wg: WaitGroup,
}

impl Manager {
    pub fn new(term: &str, pool_size: usize) -> Manager {
        Manager {
            queue: Worker::<String>::new_fifo(),
            term: term.to_owned(),
            pool_size,
            wg: WaitGroup::new(),
        }
    }

    pub fn spawn(&self) {
        for _ in 0..self.pool_size {
            let stealer = self.queue.stealer();
            let wg = self.wg.clone();

            thread::spawn(move || {
                loop {
                    if let Steal::Success(f) = stealer.steal() {
                        if f.is_empty() { break; }
                        // processing logic
                    }
                }
                drop(wg);
            });
        }
    }

    fn take(&self, file: String) {
        self.queue.push(file);
    }
    
    pub fn stop(&self) {
        for _ in 0..self.pool_size {
            self.queue.push(String::new());
        }
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