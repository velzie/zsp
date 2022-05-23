#[allow(unused_variables)]
#[allow(dead_code)]
// #[feature(once_cell)]
// use std::lazy;
use crate::parser::ExpressionFragment;
use std::collections::HashMap;

lazy_static! {
    static ref INTERRUPTS: HashMap<char, Option<Symbol>> = HashMap::from([
        ('\n', None),
        (';', None),
        (' ', None),
        ('\u{0017}',None),
        ('(', Some(Symbol::ParenStart)),
        (')', Some(Symbol::ParenEnd)),
        ('{', Some(Symbol::BlockStart)),
        ('}', Some(Symbol::BlockEnd)),
        // ('.', Symbol::If),
    ]);
    static ref KEYWORDS: HashMap<String, Symbol> = HashMap::from([
        mksym("if", Symbol::If),
        mksym("loop",Symbol::Loop),
        mksym("break",Symbol::Break),
        mksym("return",Symbol::Return),
        mksym("load",Symbol::Load),
        mksym("true",Symbol::Bool(true)),
        mksym("false",Symbol::Bool(false)),
        mksym("=", Symbol::Assign),
        mksym("else", Symbol::Else),

        mksym("<", Symbol::Op(Op::LessThan)),
        mksym("<=", Symbol::Op(Op::LessThanOrEqualTo)),
        mksym(">", Symbol::Op(Op::GreaterThan)),
        mksym(">=", Symbol::Op(Op::GreaterThanOrEqualTo)),
        mksym("==", Symbol::Op(Op::EqualTo)),
        mksym("!=", Symbol::Op(Op::NotEqualTo)),
        mksym("!", Symbol::Op(Op::Not)),

        mksym("+", Symbol::Op(Op::Plus)),
        mksym("-", Symbol::Op(Op::Minus)),
        mksym("*", Symbol::Op(Op::Multiply)),
        mksym("/", Symbol::Op(Op::Divide)),
        mksym("^", Symbol::Op(Op::Power)),
    ]);
}
// honestly if you asked me to explain what was going on here i couldn't really tell you. at least it works.
pub fn lex(inp: String) -> Vec<Token> {
    let mut chars: Vec<char> = inp.chars().collect::<Vec<char>>();
    chars.push('\u{0017}');

    let mut idx = 0;
    let mut tokens: Vec<Token> = vec![];
    let mut buf = String::default();

    while idx < chars.len() {
        let mut ch = chars[idx];

        if buf == "//" {
            loop {
                ch = chars[idx];
                if ch == '\n' || ch == '\u{0017}' {
                    buf = String::default();
                    break;
                }
                idx += 1;
            }
        }
        match ch {
            '"' => {
                let mut sbf = String::default();
                let mut depth = 1;
                while depth > 0 {
                    idx += 1;
                    ch = chars[idx];
                    if ch == '"' {
                        depth -= 1;
                    } else {
                        sbf += &ch.to_string();
                    }
                }
                tokens.push(Token {
                    symbol: Symbol::String(sbf),
                    index: idx,
                });
                buf = String::default();
            }
            _ => match INTERRUPTS.get(&ch) {
                Some(opt) => {
                    if buf.len() > 0 {
                        tokens.push(Token {
                            symbol: match KEYWORDS.get(&*buf) {
                                Some(sym) => sym.clone(),
                                None => match buf.parse::<f64>() {
                                    Ok(num) => Symbol::Number(num),
                                    Err(_) => Symbol::Name(buf),
                                },
                            },
                            index: idx,
                        });
                    }
                    match opt {
                        Some(sym) => tokens.push(Token {
                            symbol: sym.clone(),
                            index: idx,
                        }),
                        None => {}
                    }
                    buf = String::default();
                }
                None => buf.push(ch),
            },
        }
        idx += 1;
    }

    tokens
}

pub fn mksym(st: &str, sym: Symbol) -> (String, Symbol) {
    (st.to_string(), sym)
}
#[derive(Debug, Clone)]
pub struct Token {
    pub symbol: Symbol,
    pub index: usize,
}
#[derive(Debug, Clone)]
pub enum Symbol {
    Assign,
    If,
    BlockStart,
    BlockEnd,
    ParenStart,
    ParenEnd,
    Name(String),
    String(String),
    Number(f64),
    Op(Op),
    Bool(bool),
    Return,
    Loop,
    Break,
    Else,
    Load,
}
#[derive(Debug, Clone)]
pub enum Op {
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    EqualTo,
    NotEqualTo,
    Not,
}
