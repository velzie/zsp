use std::env;
use std::fs;
#[macro_use]
extern crate lazy_static;

// #![feature(once_cell)]
mod exceptions;
mod interpreter;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let contents = fs::read_to_string(&args[1]).expect("could not read file");

        let tokens = lexer::lex(contents.clone());
        println!("{:?}", tokens);
        parser::parse(tokens, contents.clone());
    } else {
        panic!("no file provided");
    }
}