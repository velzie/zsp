use crate::lexer::Logop;
use crate::lexer::Op;
use crate::lexer::Symbol;
use crate::lexer::Token;
use std::collections::HashMap;

use crate::exceptions::*;
// -> (Block, Vec<Fragment>)
pub fn parse(tkns: Vec<Token>, input: String) {
    let mut root: Block = Block {
        children: vec![],
        variables: vec![],
    };

    let mut tokens = tkns.clone();
    let funsyms = mkfunsyms(&mut tokens);

    dbg!(&funsyms);
    let functions = mkfunctions(&funsyms, &input, &root);

    dbg!(functions);

    // dbg!(blockparse(tokens, input, funsyms));

    // (blockparse(tokens, input), functions)
}
pub fn mkfunctions(
    funsyms: &HashMap<String, FunSym>,
    input: &String,
    parent: &Block,
) -> HashMap<String, Fragment> {
    let mut functions: HashMap<String, Fragment> = HashMap::new();
    for sym in funsyms {
        functions.insert(
            sym.0.to_string(),
            Fragment::Function {
                name: sym.0.to_string(),
                args: sym.1.args.clone(),
                source: blockparse(&sym.1.source, input, funsyms, parent, &sym.1.args),
            },
        );
    }
    functions
}
pub fn mkfunsyms(tokens: &mut Vec<Token>) -> HashMap<String, FunSym> {
    let mut idx = 0;
    let mut funsyms: HashMap<String, FunSym> = HashMap::new();
    while idx < tokens.len() {
        dbg!("looping");
        match tokens[idx].symbol.clone() {
            Symbol::Name(funcname) => {
                let startidx = idx;
                let mut args: Vec<String> = vec![];
                loop {
                    dbg!("looping");
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
                            let mut tkns: Vec<Token> = tokens.drain(startidx..idx + 1).collect();

                            tkns.pop();
                            tkns.drain(0..args.len() + 2);

                            idx -= idx + 1 - startidx;
                            dbg!(&tokens);
                            funsyms.insert(
                                funcname.clone(),
                                FunSym {
                                    name: funcname.to_string(),
                                    source: tkns,
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
        idx += 1;
    }
    funsyms
}

pub fn blockparse(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    args: &Vec<String>,
) -> Block {
    let mut nvs = parent.variables.clone();
    nvs.extend(args.clone());
    let mut root: Block = Block {
        children: vec![],
        variables: nvs,
    };

    let mut idx = 0;

    while idx < tokens.len() {
        let mut token = &tokens[idx];
        match &token.symbol {
            Symbol::If => {

                // root.children.push()
            }
            Symbol::Assign => {}
            // Symbol::BlockStart => {}
            // Symbol::BlockEnd => {}
            Symbol::Name(name) => match funsyms.get(name) {
                Some(fnsym) => {
                    let mut args: Vec<Vec<ExpressionFragment>> = vec![];

                    // keep running this code until every required argument has been captured
                    while args.len() != fnsym.args.len() {
                        let mut exp: Vec<ExpressionFragment> = vec![];
                        // the buffer for the current argument
                        loop {
                            idx += 1;
                            if idx == tokens.len() {
                                break;
                            }
                            // dbg!(&tokens[idx]);
                            // dbg!(&exp);
                            // dbg!(args.len(), fnsym.args.len());
                            // dbg!(idx);
                            token = &tokens[idx];
                            match token.symbol.clone() {
                                Symbol::String(_) | Symbol::Number(_) | Symbol::Name(_) => {
                                    if exp.len() == 0
                                        || matches!(
                                            exp.last().unwrap_or(&ExpressionFragment::Logop(
                                                Logop::EqualTo
                                            )), //dumb hack please fix
                                            ExpressionFragment::Logop(_)
                                        )
                                        || matches!(exp.last().unwrap(), ExpressionFragment::Op(_))
                                    {
                                        // if it's the first element in the array or part of an expression
                                        match token.symbol.clone() {
                                            Symbol::String(s) => exp
                                                .push(ExpressionFragment::Value(Value::String(s))),
                                            Symbol::Number(n) => exp
                                                .push(ExpressionFragment::Value(Value::Number(n))),
                                            Symbol::Name(n) => {
                                                exp.push(ExpressionFragment::Name(n))
                                            }
                                            _ => panic!(),
                                        }
                                    } else {
                                        // otherwise jump to the next argument
                                        if args.len() < fnsym.args.len() {
                                            dbg!("jumping to next arg");
                                            idx -= 1;
                                            break;
                                        } else {
                                            unexpected_symbol_exception(
                                                input.clone(),
                                                token.index,
                                                Fragment::Block(root.clone()),
                                                token.symbol.clone(),
                                            );
                                        }
                                    }
                                }
                                Symbol::Logop(_) | Symbol::Op(_) => {}
                                _ => unexpected_symbol_exception(
                                    input.clone(),
                                    token.index,
                                    Fragment::Block(root.clone()),
                                    token.symbol.clone(),
                                ),
                            }
                        }
                        args.push(exp);
                        // why was this commented out
                    }
                    root.children.push(Fragment::Call {
                        name: name.to_string(),
                        args: args,
                    })
                    // idx -= 1;
                }
                None => unexpected_name_exception(input.clone(), token.index, token.symbol.clone()),
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
    variables: Vec<String>,
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
        value: Value,
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
    Value(Value),
}
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    // Bool(bool),
}
