#[derive(Debug)]
pub struct HS<'a, 'b> {
    dir: String,
    sc: &'a Scanner,
    mg: &'b Manager,
}

impl<'a, 'b> HS<'a, 'b> {
    fn new(dir: &'d str, sc: &'a Scanner, &'b Manager) -> HS<'a, 'b> {
        HS {
            dir: dir.to_owned(),
            sc,
            mg,
        }
    }
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