use std::fs::OpenOptions;

const GIT_HASH: &str = env!("GIT_HASH");

fn build_version_str() -> String {
    let mut hash = String::new();

    if !GIT_HASH.is_empty() {
        hash.push_str("-");
        hash.push_str(GIT_HASH);
    }
    format!("v{}{}", env!("CARGO_PKG_VERSION"), hash)
}