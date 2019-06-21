extern crate walkdir;

use walkdir::WalkDir;
use std::fs::File;
use std::io::Read;
use std::io;

// These constants will be provided by the CLI later.
const DIR: &str = "./";

fn main() -> Result<(), io::Error> {
    let mut buf: Vec<u8> = Vec::new();

    for item in WalkDir::new(DIR).into_iter().filter_map(|i| i.ok()) {
        if item.file_type().is_file() {
            let mut handle = File::open(item.path())?;

            buf.clear();
            handle.read_to_end(&mut buf)?;

            println!("opened {}", item.path().display().to_string());
        }
    }
    Ok(())
}
