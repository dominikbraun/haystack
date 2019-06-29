use std::fs::File;
use std::io;
use std::io::{BufReader, Error, Read};
use std::path::Path;
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
            let term = self.term.clone();
            let wg = self.wg.clone();

            thread::spawn(move || {
                loop {
                    if let Steal::Success(f) = stealer.steal() {
                        if f.is_empty() { break; }

                        let handle = match File::open(Path::new(&f)) {
                            Ok(f) => f,
                            Err(err) => {
                                println!("{}", err);
                                continue;
                            }
                        };
                        let occurences = process(&term, handle);

                        if occurences > 0 {
                            println!("Found in {}", f);
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