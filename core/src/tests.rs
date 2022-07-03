use colored::Colorize;

use crate::runtime::run;
use std::{fs, path::Path};

// #[test]
pub fn tests() {
    // std::env::set_current_dir(Path::new("../"));
    let tests = fs::read_dir("tests").unwrap();

    for path in tests {
        println!(
            "{}",
            format!(
                "{} {}",
                "Running test file".bold().truecolor(255, 255, 255),
                path.as_ref()
                    .unwrap()
                    .path()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .blue()
            )
        );
        run(path.unwrap().path().as_path());
        println!("{}", "passed test!".green());
    }

    println!("passed tests!");
}
