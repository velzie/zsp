use crate::lexer::Symbol;
use crate::parser::Block;
use colored::Colorize;
use std::io;

pub fn unexpected_symbol_exception(input: &String, idx: usize, context: Block, symbol: Symbol) {
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
pub fn rtexception(input: &String, idx: usize, errtype: &str, message: &str) {
    exception(
        input,
        idx,
        errtype,
        &format!("{}RUNTIME EXCEPTION: {}", "", message), //color::Fg(color::Blue)
    );
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

    println!("{}", "-".repeat(offset + 9 + errtype.len()).red());
    println!(
        "      \"{}\"     {}",
        allines[lines].truecolor(255, 255, 255),
        format!(
            "at line {}, col {}",
            lines.to_string().truecolor(255, 255, 255),
            offset.to_string().truecolor(255, 255, 255)
        )
    );
    println!(
        "      {}{}      {}\n{} {}",
        " ".repeat(offset - 1),
        "^".bright_red(),
        errtype.purple().bold(),
        "ERROR:".red().bold(),
        message.bright_purple().bold()
    );
    println!("{}", "-".repeat(offset + 9 + errtype.len()).red());
    panic!();
}
