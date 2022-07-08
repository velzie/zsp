use colored::Colorize;

use crate::runtime::execute;
use std::fs;

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
        let contents = fs::read_to_string(path.unwrap().path().as_path())
            .expect("could not read file")
            .chars()
            .filter(|c| c != &'\r')
            .collect::<String>();
        execute(contents, None);
        println!("{}", "passed test!".green());
    }

    println!("passed tests!");
}
