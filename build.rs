use std::process::Command;

fn main() {
    // get current git commit hash
    let git_hash = match Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output() {
        Ok(hash_out) => String::from_utf8(hash_out.stdout),
        Err(_) => Result::Ok(String::new())
    };

    let git_hash = git_hash.unwrap_or(String::new());
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}