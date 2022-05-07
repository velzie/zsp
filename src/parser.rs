use crate::lexer::Logop;
use crate::lexer::Op;
use crate::lexer::Symbol;
use crate::lexer::Token;
use std::collections::HashMap;

use crate::exceptions::*;
// -> (Block, Vec<Fragment>)
pub fn parse(tkns: Vec<Token>, input: String) {
    let mut tokens = tkns.clone();
    let funsyms = mkfunsyms(tokens);
    dbg!(&funsyms);
    let functions = mkfunctions(funsyms, input);

    // (blockparse(tokens, input), functions)
}
pub fn mkfunctions(funsyms: HashMap<String, FunSym>, input: String) -> HashMap<String, Fragment> {
    let mut functions: HashMap<String, Fragment> = HashMap::new();
    for sym in funsyms.clone() {
        functions.insert(
            sym.0.clone(),
            Fragment::Function {
                name: sym.0,
                args: sym.1.args,
                source: blockparse(sym.1.source, input.clone(), funsyms.clone()),
            },
        );
    }
    functions
}
pub fn mkfunsyms(mut tokens: Vec<Token>) -> HashMap<String, FunSym> {
    let mut idx = 0;
    let mut funsyms: HashMap<String, FunSym> = HashMap::new();
    while idx < tokens.len() {
        match tokens[idx].symbol.clone() {
            Symbol::Name(funcname) => {
                let startidx = idx;
                let mut args: Vec<String> = vec![];
                loop {
                    idx += 1;
                    match &tokens[idx].symbol.clone() {
                        Symbol::BlockStart => {
                            let mut depth = 1;
                            while depth > 0 {
                                idx += 1;
                                match &tokens[idx].symbol.clone() {
                                    Symbol::BlockStart => depth += 1,
                                    Symbol::BlockEnd => depth -= 1,
                                    _ => {}
                                }
                            }
                            funsyms.insert(
                                funcname,
                                FunSym {
                                    name: funcname.to_string(),
                                    source: tokens.drain(startidx + 2 + args.len()..idx).collect(),
                                    args: args,
                                },
                            );
                            break;
                        }
                        Symbol::Name(arg) => args.push(arg.to_string()),
                        _ => break,
                    }
                }
            }
            _ => {}
        }
    }
    funsyms
}

pub fn blockparse(tokens: Vec<Token>, input: String, funsyms: HashMap<String, FunSym>) -> Block {
    let mut root: Block = Block { children: vec![] };

    let mut idx = 0;

    while idx < tokens.len() {
        let token = &tokens[idx];
        match &token.symbol {
            Symbol::If => {

                // root.children.push()
            }
            Symbol::Assign => {}
            // Symbol::BlockStart => {}
            // Symbol::BlockEnd => {}
            Symbol::Name(name) => match funsyms.get(name) {
                Some(fnsym) => {
                    let args: Vec<Vec<ExpressionFragment>> = vec![];

                    while args.len() != fnsym.args.len() {
                        let exp: Vec<ExpressionFragment> = vec![];
                        loop {
                            idx += 1;
                            match tokens[idx].symbol.clone() {
                                Symbol::String(s) => {}
                                Symbol::Number(n) => {}

                                Symbol::Logop(l) => {}
                                Symbol::Op(o) => {}
                                Symbol::Name(n) => {}
                                _ => unexpected_symbol_exception(
                                    input.clone(),
                                    token.index,
                                    Fragment::Block(root.clone()),
                                    token.symbol.clone(),
                                ),
                            }
                        }
                        // args.push(exp);
                    }
                    root.children.push(Fragment::Call {
                        name: name.to_string(),
                        args: args,
                    })
                    // idx -= 1;
                }
                None => unexpected_symbol_exception(
                    input.clone(),
                    token.index,
                    Fragment::Block(root.clone()),
                    token.symbol.clone(),
                ),
            },
            _ => unexpected_symbol_exception(
                input.clone(),
                token.index,
                Fragment::Block(root.clone()),
                token.symbol.clone(),
            ),
        }

        idx += 1;
    }
    root
}

#[derive(Debug, Clone)]
pub struct Block {
    children: Vec<Fragment>,
}
#[derive(Debug, Clone)]
pub struct FunSym {
    name: String,
    source: Vec<Token>,
    args: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum Fragment {
    Block(Block),
    If {
        condition: Vec<ExpressionFragment>,
        trueblock: Block,
        falseblock: Block,
    },
    Function {
        name: String,
        args: Vec<String>,
        source: Block,
    },
    Assignment {
        name: String,
        value: Variable,
    },
    Call {
        name: String,
        args: Vec<Vec<ExpressionFragment>>,
    },
    InternalCall {
        name: String,
        args: Vec<Vec<ExpressionFragment>>,
    },
}
#[derive(Debug, Clone)]
pub enum ExpressionFragment {
    Name(String), //remember that this could also be a function call :/
    Op(Op),
    Logop(Logop),
}

#[derive(Debug, Clone)]
pub enum Variable {
    String(String),
    Number(f64),
    Bool(bool),
}
