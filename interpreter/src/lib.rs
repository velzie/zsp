#![feature(strict_provenance)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
pub mod builtins;
pub mod exceptions;
pub mod lexer;
pub mod libp;
pub mod parser;
pub mod runtime;
