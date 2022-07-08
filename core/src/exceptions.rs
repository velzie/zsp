use crate::lexer::Symbol;
use crate::parser::Block;
use colored::Colorize;

pub fn unexpected_symbol_exception(
    input: &String,
    idx: usize,
    symbol: Symbol,
    allowed: Vec<Symbol>,
) {
    exception(
        input,
        idx,
        "UnexpectedSymbolException",
        &format!(
            "Expected {}: {}",
            if allowed.len() > 1 {
                "one of the following symbols"
            } else {
                "the symbol"
            },
            allowed
                .iter()
                .fold(String::new(), |acc, x| acc + ", " + &x.display_name())
        ),
    );
}

pub fn old_unexpected_symbol_exception(input: &String, idx: usize, context: Block, symbol: Symbol) {
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
    exceptionbuilder(input, idx, errtype, message);
}
pub fn exceptionbuilder(input: &String, idx: usize, errtype: &str, message: &str) {
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
    let allines: Vec<&str> = input.lines().collect();

    let line1 = format!(
        "      \"{}\"     {}",
        allines[lines].truecolor(255, 255, 255),
        format!(
            "at line {}, col {}",
            lines.to_string().truecolor(255, 255, 255),
            offset.to_string().truecolor(255, 255, 255)
        )
    );
    let line2 = format!(
        "      {}{}      {}",
        " ".repeat(offset - 1),
        "^".bright_red(),
        errtype.purple().bold(),
    );
    let line3 = format!(
        "{} {}",
        "ERROR:".red().bold(),
        message.bright_purple().bold()
    );

    let dasheslen = line3.len() / 2;
    // dbg!(dasheslen);
    println!("{}", "-".repeat(dasheslen).red());
    println!("{}", line1);
    println!("{}", line2);
    println!("{}", line3);
    println!("{}", "-".repeat(dasheslen).red());
    panic!()
}
