#![feature(strict_provenance)]
use std::env;
use std::fs;
use std::path::Path;
#[macro_use]
extern crate lazy_static;

// #![feature(once_cell)]
pub mod exceptions;
pub mod lexer;
pub mod libp;
pub mod parser;
pub mod runtime;
// #[cfg(test)]
mod builtins;
mod tests;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        runtime::run(Path::new(&args[1]));
    } else {
        tests::tests();
        // panic!("no file provided");
    }
}
