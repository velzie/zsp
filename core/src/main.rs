use std::env;
use std::path::Path;

#[macro_use]
extern crate lazy_static;

// #[macro_use]
pub mod utils;
// #![feature(once_cell)]
pub mod exceptions;
pub mod lexer;
pub mod parser;
pub mod runtime;
// #[cfg(test)]
pub mod builtins;
mod tests;
fn main() {
    // utils::l
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        runtime::run(Path::new(&args[1]));
    } else {
        tests::tests();
        // panic!("no file provided");
    }
}
