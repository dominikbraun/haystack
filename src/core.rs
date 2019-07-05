extern crate crossbeam;
extern crate walkdir;

use std::fs;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use crossbeam::deque::Injector;
use crossbeam::deque::Steal;
use walkdir::WalkDir;

use crate::Settings;

pub struct Manager<'s> {
    term: String,
    opt: &'s Settings,
    queue: Arc<Injector<String>>,
    done_tx: crossbeam::Sender<u32>,
    done_rx: crossbeam::Receiver<u32>,
}

impl<'s> Manager<'s> {
    pub fn new(term: &str, options: &'s Settings) -> Manager<'s> {
        let (done_tx, done_rx) = crossbeam::bounded(options.pool_size);

        Manager {
            term: term.to_owned(),
            opt: options,
            queue: Arc::new(Injector::<String>::new()),
            done_tx,
            done_rx,
        }
    }

    pub fn spawn(&self) -> bool {
        for _ in 0..self.opt.pool_size {
            let term = self.term.clone();
            let queue = Arc::clone(&self.queue);
            let buf_size = self.opt.buf_size;
            let done_tx = self.done_tx.clone();
            let case_insensitive = self.opt.case_insensitive;

            let mut stdout = BufWriter::new(io::stdout());

            thread::spawn(move || {
                let mut found: u32 = 0;

                loop {
                    if let Steal::Success(f) = queue.steal() {
                        if f.is_empty() {
                            // Leave the loop since an empty string is
                            // the stop signal for worker queues.
                            break;
                        }
                        let path = Path::new(&f);

                        let mut handle = match fs::File::open(path) {
                            Ok(handle) => handle,
                            Err(e) => {
                                eprintln!("Error occurred while reading file {}: {}", &f, e);
                                continue;
                            },
                        };

                        let val = process(&term, &mut handle, buf_size, case_insensitive);

                        if val > 0 {
                            found += val;

                            let mut output = String::with_capacity(2048);

                            output.push_str(&val.to_string());
                            output.push_str("x in ");
                            output.push_str(&f);
                            output.push('\n');

                            stdout.write_all(output.as_bytes()).unwrap_or_else(|err| {
                                eprintln!("{}", err);
                            });
                        }
                    }
                }
                stdout.flush().unwrap_or_else(|err| {
                    eprintln!("{}", err);
                });
                ;

                done_tx.send(found).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                });
            });
        }
        true
    }

    fn take(&self, file: String) {
        self.queue.push(file);
    }

    pub fn stop(&self) -> u32 {
        // Send an empty string to each worker queue.
        for _ in 0..self.opt.pool_size {
            self.queue.push(String::new());
        }

        let mut sum: u32 = 0;

        for _ in 0..self.opt.pool_size {
            sum += self.done_rx.recv().unwrap_or(0);
        }
        sum
    }
}

pub fn scan(dir: &str, manager: &Manager) -> Result<(), io::Error> {
    let mut walker = WalkDir::new(dir.to_owned());

    if manager.opt.max_depth.is_some() {
        walker = walker.max_depth(manager.opt.max_depth.unwrap());
    }

    let items = walker.into_iter().filter_map(|i| {
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

fn process(term: &str, handle: &mut dyn Read, buf_size: usize, case_insensitive: bool) -> u32 {
    let mut buf: Vec<u8> = vec![0; buf_size];

    let mut cursor = 0;
    let mut found: u32 = 0;
    let term = term.as_bytes();

    loop {
        if let Ok(len) = handle.read(&mut buf) {
            if len == 0 {
                break;
            }

            for val in buf.iter().take(len) {
                let val = if case_insensitive {
                    (*val).to_ascii_lowercase()
                } else {
                    *val
                };

                if val == term[cursor] {
                    cursor += 1;
                } else if cursor > 0 {
                    if val == term[0] {
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
    use std::io::{BufReader, Cursor, Seek, SeekFrom, Write};

    use crate::core::process;

    fn setup_fake_file(data: &str) -> Cursor<Vec<u8>> {
        let mut fake_file = Cursor::new(Vec::new());

        // Write into the "file" and seek to the beginning
        fake_file.write_all(data.as_bytes()).unwrap();
        fake_file.seek(SeekFrom::Start(0)).unwrap();

        return fake_file
    }

    #[test]
    fn find_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        // use buf size 4 to test also, if it works if the buffer is not full at the end
        assert_eq!(1, process("789", &mut reader, 4, false), "finding the search term at the end should return true");
    }

    /// This test should NOT fail (e. g. index out of bounds)
    #[test]
    fn find_only_half_at_end() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(0, process("8910", &mut reader, 5, false), "finding the pattern only half at the end should return false");
    }

    #[test]
    fn find_at_beginning() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(1, process("012", &mut reader, 5, false), "finding the pattern at the beginning should return true");
    }

    #[test]
    fn find_at_center() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(1, process("34567", &mut reader, 5, false), "finding the pattern at the center should return true");
    }

    #[test]
    fn finding_nothing() {
        let mut reader = BufReader::new(setup_fake_file("0123456789"));
        assert_eq!(0, process("asdf", &mut reader, 5, false), "finding nothing should return false");
    }

    #[test]
    fn find_several_times() {
        let mut reader = BufReader::new(setup_fake_file("abc01234abc56789abcjab"));
        assert_eq!(3, process("abc", &mut reader, 10, false), "the pattern should exist 3 times in the file");
    }

    #[test]
    fn find_case_insensitive() {
        let mut reader = BufReader::new(setup_fake_file("ABC01234aBc56789abcjab"));
        assert_eq!(3, process("abc", &mut reader, 10, true), "the pattern should exist 3 times in the file");
    }

    #[test]
    fn find_not_case_insensitive() {
        let mut reader = BufReader::new(setup_fake_file("abc01234abc56789abcjab"));
        assert_eq!(0, process("ABC", &mut reader, 10, false), "the pattern should exist 3 times in the file");
    }
}