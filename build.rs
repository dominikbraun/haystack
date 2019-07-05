use std::process::Command;

fn main() {
    // get current git commit hash
    let git_hash = match Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output() {
        Ok(hash_out) => String::from_utf8(hash_out.stdout),
        Err(_) => Result::Ok(String::new())
    };

    let git_hash = git_hash.unwrap_or(String::new());

    let mut hash = String::new();

    if !git_hash.is_empty() {
        hash.push_str("-");
        hash.push_str(git_hash.as_str());
    }

    println!("cargo:rustc-env=FULL_VERSION={}", format!("v{}{}", env!("CARGO_PKG_VERSION"), hash));
}