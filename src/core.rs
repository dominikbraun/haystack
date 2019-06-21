#[derive(Debug)]
pub struct HS<'a, 'b> {
    sc: &'a Scanner,
    mg: &'b Manager,
}

#[derive(Debug, Copy, Clone)]
struct Scanner {}

impl Scanner {
    fn run(&self, path: &str) {
        unimplemented!();
    }
}

#[derive(Debug)]
struct Manager {
    pool: Vec<Worker>,
}

#[derive(Debug, Copy, Clone)]
struct Worker {}