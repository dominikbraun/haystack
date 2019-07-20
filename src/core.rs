extern crate crossbeam;
extern crate walkdir;

use std::fs;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crossbeam::deque::Injector;
use crossbeam::deque::Steal;
use walkdir::WalkDir;

use crate::Params;

// Manager provides methods to start and manage all worker threads.
// It also holds the CSP channels for communication with the workers
// and provides the global injector queue for new tasks.
pub struct Manager<'p> {
    args: &'p Params,
    queue: Arc<Injector<String>>,
    done_tx: crossbeam::Sender<u32>,
    done_rx: crossbeam::Receiver<u32>,
}

impl<'p> Manager<'p> {
    // Creates a new instance of Manager with the given CLI args.
    // At the moment, a buffered channel is being used.
    pub fn new(args: &'p Params) -> Manager<'p> {
        let (done_tx, done_rx) = crossbeam::bounded(args.pool_size);

        Manager {
            args,
            queue: Arc::new(Injector::<String>::new()),
            done_tx,
            done_rx,
        }
    }

    // Spawns n new worker threads (using n = pool_size). All needed
    // values are copied/cloned and then moved into the thread.
    pub fn spawn(&self) -> bool {
        for _ in 0..self.args.pool_size {
            
            let term = self.args.needle.clone();
            let queue = Arc::clone(&self.queue);
            let buf_size = self.args.buf_size;
            let done_tx = self.done_tx.clone();
            let case_insensitive = self.args.case_insensitive;

            // Wraps an instance of BufWriter around stdout so that
            // stdout will only be flushed when the buffer is full.
            let mut stdout = BufWriter::new(io::stdout());

            thread::spawn(move || {
                let mut found: u32 = 0;

                loop {
                    if let Steal::Success(f) = queue.steal() {
                        if f.is_empty() {
                            // Leave the loop since an empty string is
                            // the stop signal for worker threads.
                            break;
                        }
                        let path = Path::new(&f);

                        let mut handle = match fs::File::open(path) {
                            Ok(handle) => handle,
                            Err(e) => {
                                eprintln!("{}", e);
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

                // Confirm that the thread has finished. On that occasion
                // the total number of matches will be used as a value.
                done_tx.send(found).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                });
            });
        }
        true
    }

    // Pushes a new file (represented by a String) into the queue
    // which will be popped of by a worker.
    fn take(&self, file: String) {
        self.queue.push(file);
    }

    // Will be called after scan() has finished. This function sends
    // a stop signal to all workers and waits until they've finished.
    pub fn stop(&self) -> u32 {
        for _ in 0..self.args.pool_size {
            self.queue.push(String::new());
        }

        (0..self.args.pool_size)
            .filter_map(|_| self.done_rx.recv().ok())
            .sum()
    }
}

// Scans a given directory recursively. Each file will be pushed
// into the global injector queue provided by the Manager. Returning
// a Result will indicate that the Manager may stop the workers.
pub fn scan(dir: &PathBuf, manager: &Manager) -> Result<(), io::Error> {
    let mut walker = WalkDir::new(dir);
    // get whitelist and convert it to lower case
    let whitelist: Vec<String> = manager.args.whitelist.iter().map(|file_ending| file_ending.to_lowercase()).collect();

    if manager.args.max_depth.is_some() {
        let d = manager.args.max_depth.unwrap();
        walker = walker.max_depth(d);
    }

    let items = walker.into_iter().filter_map(|i| {
        i.ok()
    }).filter(|dir_entry| {
        if !whitelist.is_empty() {
            // filter for whitelist
            let file_ending: Option<Option<&str>> = dir_entry.file_name().to_str().map(|ending| ending.split('.').last());
            // todo: better unwrap code...
            return if file_ending.is_some() && file_ending.unwrap().is_some() {
                let name = file_ending.unwrap().unwrap().to_lowercase();
                whitelist.iter().any(|whitelisted_ending| whitelisted_ending == &name)
            } else {
                false
            }
        }

        true
    });

    for i in items {
        if i.file_type().is_file() {
            let path = i.path().display().to_string();
            manager.take(path);
        }
    }
    Result::Ok(())
}

// Searches a file represented by handle for a given term by performing
// a byte-wise comparison. In order to keep the memory footprint small
// even with many threads, the file will be merely loaded as byte chunks.
fn process<T: Read>(term: &str, handle: &mut T, buf_size: usize, case_insensitive: bool) -> u32 {
    let mut buf: Vec<u8> = vec![0; buf_size];

    let mut cursor = 0;
    let mut found: u32 = 0;
    let term = term.as_bytes();

    // Only process the byte buffer if the file bytes have been
    // read from the handle successfully. Otherwise, end the loop.
    while let Ok(len) = handle.read(&mut buf) {
        if len == 0 {
            break;
        }

        for val in buf.iter().take(len) {
            let val = if case_insensitive {
                (*val).to_ascii_lowercase()
            } else {
                *val
            };

            // The fact that a matching result may be splitted into two pieces
            // (because the buffer is emptied and refilled chunk by chunk) doesn't
            // matter since the cursor position remains the same.
            if val == term[cursor] {
                cursor += 1;
            } else if cursor > 0 {
                if val == term[0] {
                    cursor = 1;
                } else {
                    cursor = 0;
                }
            }

            // A matching result was found if the cursor reaches the term's end.
            if cursor == term.len() {
                found += 1;
                cursor = 0;
            }
        }
    };
    found
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor, Seek, SeekFrom, Write};

    use crate::core::process;

    fn dummy_file(data: &str) -> Cursor<Vec<u8>> {
        let mut file = Cursor::new(Vec::new());

        // Write the test data into the dummy file and seek to the beginning.
        file.write_all(data.as_bytes()).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        file
    }

    #[test]
    fn find_at_end() {
        let mut reader = BufReader::new(dummy_file("0123456789"));
        // use buf size 4 to test also, if it works if the buffer is not full at the end
        assert_eq!(1, process("789", &mut reader, 4, false), "finding the search term at the end should return true");
    }

    /// This test should NOT fail (e. g. index out of bounds)
    #[test]
    fn find_only_half_at_end() {
        let mut reader = BufReader::new(dummy_file("0123456789"));
        assert_eq!(0, process("8910", &mut reader, 5, false), "finding the pattern only half at the end should return false");
    }

    #[test]
    fn find_at_beginning() {
        let mut reader = BufReader::new(dummy_file("0123456789"));
        assert_eq!(1, process("012", &mut reader, 5, false), "finding the pattern at the beginning should return true");
    }

    #[test]
    fn find_at_center() {
        let mut reader = BufReader::new(dummy_file("0123456789"));
        assert_eq!(1, process("34567", &mut reader, 5, false), "finding the pattern at the center should return true");
    }

    #[test]
    fn finding_nothing() {
        let mut reader = BufReader::new(dummy_file("0123456789"));
        assert_eq!(0, process("asdf", &mut reader, 5, false), "finding nothing should return false");
    }

    #[test]
    fn find_several_times() {
        let mut reader = BufReader::new(dummy_file("abc01234abc56789abcjab"));
        assert_eq!(3, process("abc", &mut reader, 10, false), "the pattern should exist 3 times in the file");
    }

    #[test]
    fn find_case_insensitive() {
        let mut reader = BufReader::new(dummy_file("ABC01234aBc56789abcjab"));
        assert_eq!(3, process("abc", &mut reader, 10, true), "the pattern should exist 3 times in the file");
    }

    #[test]
    fn find_not_case_insensitive() {
        let mut reader = BufReader::new(dummy_file("abc01234abc56789abcjab"));
        assert_eq!(0, process("ABC", &mut reader, 10, false), "the pattern should exist 3 times in the file");
    }
}
