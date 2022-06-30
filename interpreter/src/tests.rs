use crate::runtime::run;
use std::{fs, path::Path};

// #[test]
pub fn tests() {
    // std::env::set_current_dir(Path::new("../"));
    let tests = fs::read_dir("tests").unwrap();

    for path in tests {
        run(path.unwrap().path().as_path());
    }

    println!("passed tests!");
}
