use crate::lexer::Symbol;
use crate::parser::Fragment;
use std::io;
use termion::{clear, color, style};

pub fn unexpected_symbol_exception(input: &String, idx: usize, context: Fragment, symbol: Symbol) {
    exception(
        input,
        idx,
        "UnexpectedSymbolException",
        &format!("Symbol {:?} cannot appear in context {:?}", symbol, context),
    )
}
pub fn unexpected_name_exception(input: &String, idx: usize, symbol: Symbol) {
    exception(
        input,
        idx,
        "UnexpectedNameException",
        &format!("Name {:?} is undefined ", symbol),
    )
}
pub fn exception(input: &String, idx: usize, errtype: &str, message: &str) {
    let mut i = 0;
    let mut lines = 0;
    let mut offset = 0;
    while i < idx {
        if input.chars().nth(i).unwrap() == '\n' {
            lines += 1;
            offset = 0;
        }
        offset += 1;
        i += 1;
    }
    let allines: Vec<&str> = input.split('\n').collect();
    println!(
        "\n{}      {}\"{}\"{}{}     at line {}, col {}",
        color::Fg(color::Red),
        color::Bg(color::Black),
        allines[lines],
        color::Bg(color::Reset),
        color::Fg(color::Blue),
        lines,
        offset
    );
    println!(
        "{}{}ERROR:{}{}{} ^       {}",
        color::Bg(color::Cyan),
        style::Bold,
        color::Fg(color::Reset),
        color::Bg(color::Reset),
        style::Reset,
        errtype
    );
    println!(
        "\n{}{}          {}\n",
        color::Fg(color::Green),
        style::Bold,
        message
    );
}
