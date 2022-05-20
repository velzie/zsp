//TODO : replace the clones with borrows and replace the strings with &strs
// Split the argparser into a function since it's called several times

use crate::lexer::Logop;
use crate::lexer::Op;
use crate::lexer::Symbol;
use crate::lexer::Token;
use std::collections::HashMap;
use std::hash::Hash;
use std::process::id;
use std::vec;

use crate::exceptions::*;
// -> (Block, Vec<Fragment>)
pub fn parse(tkns: Vec<Token>, input: String) -> Root {
    let mut tokens = tkns.clone();
    let mut funsyms = make_funsyms(&mut tokens, &input);

    funsyms.insert(
        String::from("print"),
        FunSym {
            name: String::from("print"),
            args: vec![String::from("text")],
            source: vec![],
        },
    );

    dbg!(&funsyms);

    let rootblock = parse_block(&tokens, &input, &funsyms, None, &vec![], 0, tokens.len());

    let functions = make_functions(&funsyms, &input, &rootblock);
    // substitute vec![] for global constants later
    Root {
        root: rootblock,
        functions,
    }
}
pub fn make_functions(
    funsyms: &HashMap<String, FunSym>,
    input: &String,
    scope: &Block,
) -> HashMap<String, Fragment> {
    let mut functions: HashMap<String, Fragment> = HashMap::new();
    for sym in funsyms {
        functions.insert(
            sym.0.to_string(),
            Fragment::Function {
                name: sym.0.to_string(),
                args: sym.1.args.clone(),
                source: parse_block(
                    &sym.1.source,
                    input,
                    funsyms,
                    Some(scope),
                    &sym.1.args,
                    0,
                    sym.1.source.len(),
                ),
            },
        );
    }
    functions
}
pub fn make_funsyms(tokens: &mut Vec<Token>, input: &String) -> HashMap<String, FunSym> {
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
                            idx = next_symbol_block(
                                &tokens,
                                &input,
                                idx,
                                Symbol::BlockStart,
                                Symbol::BlockEnd,
                            );
                            let mut tkns: Vec<Token> = tokens.drain(startidx..idx + 1).collect(); //drains the tokens. messy but i can't think of a better way of doing this
                            tkns.pop();
                            tkns.drain(0..args.len() + 2);
                            idx = startidx;
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
                        Symbol::Name(arg) => {
                            if arg == &funcname {
                                // assume no arguments
                                break;
                            }
                            args.push(arg.to_string())
                        }
                        _ => break,
                    }
                }
            }
            Symbol::If => {
                idx += 1;
                // let endx =
                // next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);
                let endx = next_symbol(&tokens, &input, idx, Symbol::BlockEnd);
                //WARNGIN : WILL ACT WEIRDLY ON NESTED IF BLOCKS. SHOULDN"T MATTER
                idx = endx;
            }
            _ => idx += 1,
        }
    }
    funsyms
}

/// EXPECTED BEHAVIOR: Start idx just before the expression, idx will end up after the expression
pub fn parse_args(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    scope: &Block,
    exargs: &Vec<String>,
    argslen: usize,
    idx: &mut usize,
) -> Vec<Vec<ExpressionFragment>> {
    dbg!(argslen);
    let start = idx.clone();
    let mut args: Vec<Vec<ExpressionFragment>> = vec![];

    // keep running this code until every required argument has been captured
    while args.len() != argslen {
        let mut exp: Vec<ExpressionFragment> = vec![];

        // the buffer for the current argument
        loop {
            if *idx >= tokens.len() {
                break;
            }
            let token = &tokens[*idx];
            match &token.symbol {
                Symbol::String(_) | Symbol::Number(_) | Symbol::Name(_) => {
                    if exp.len() == 0
                        || matches!(
                            exp.last()
                                .unwrap_or(&ExpressionFragment::Logop(Logop::EqualTo)), //dumb hack please fix
                            ExpressionFragment::Logop(_)
                        )
                        || matches!(exp.last().unwrap(), ExpressionFragment::Op(_))
                    {
                        // if it's the first element in the array or part of an expression
                        match token.symbol.clone() {
                            Symbol::String(s) => {
                                exp.push(ExpressionFragment::Constant(Constant::String(s)))
                            }
                            Symbol::Number(n) => {
                                exp.push(ExpressionFragment::Constant(Constant::Number(n)))
                            }
                            Symbol::Name(n) => {
                                // make sure to check if the name is valid
                                if scope.variables.contains(&n) {
                                    exp.push(ExpressionFragment::Name(n));
                                } else if funsyms.contains_key(&n) {
                                    let fnsym = funsyms.get(&n).unwrap();
                                    dbg!(&idx);
                                    exp.push(parse_fncall(
                                        &tokens, &input, &funsyms, &scope, &exargs, &mut *idx,
                                        &fnsym,
                                    ));
                                } else {
                                    unexpected_name_exception(&input, token.index, Symbol::Name(n));
                                    panic!();
                                }
                            }
                            _ => panic!(),
                        }
                    } else {
                        // otherwise jump to the next argument
                        if args.len() < argslen {
                            *idx -= 1;
                            break;
                        } else {
                            unexpected_symbol_exception(
                                &input,
                                token.index,
                                Fragment::Block(scope.clone()),
                                token.symbol.clone(),
                            );
                        }
                    }
                }
                Symbol::Logop(l) => exp.push(ExpressionFragment::Logop(l.clone())),
                Symbol::Op(o) => exp.push(ExpressionFragment::Op(o.clone())),
                _ => unexpected_symbol_exception(
                    &input,
                    token.index,
                    Fragment::Block(scope.clone()),
                    token.symbol.clone(),
                ),
            }
        }
        if exp.len() > 0 {
            args.push(exp);
        } else {
            break;
        }
        *idx += 1;

        // why was this commented out
    }
    if args.len() < argslen {
        exception(
            &input,
            tokens[start].index,
            "ArgumentException",
            "Not enough arguments!",
        );
    }
    args
}

// panic!(split the arguments parser into a separate function);

// returns the index of the next symbol
pub fn next_symbol(tokens: &Vec<Token>, input: &String, start: usize, end: Symbol) -> usize {
    let mut idx = start;
    loop {
        let token = &tokens[idx];
        if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&end) {
            return idx;
        }
        idx += 1;
        if idx == tokens.len() {
            exception(
                &input,
                idx,
                "EOFexcpetion",
                &format!("Expected to find {:?}, got EOF instead", &end),
            );
            panic!();
        }
    }
}
pub fn next_symbol_block(
    tokens: &Vec<Token>,
    input: &String,
    start: usize,
    addepth: Symbol,
    backdepth: Symbol,
) -> usize {
    let mut idx = start;
    let mut depth = 1;
    loop {
        idx += 1;
        if idx == tokens.len() {
            exception(
                &input,
                idx,
                "EOFexcpetion",
                &format!("Expected to find {:?}, got EOF instead", &backdepth),
            );
            panic!();
        }
        let token = &tokens[idx];
        if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&addepth) {
            depth += 1;
        } else if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&backdepth) {
            depth -= 1;
            if depth == 0 {
                return idx;
            }
        }
    }
}

pub fn parse_block(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: Option<&Block>,
    args: &Vec<String>,
    idxstart: usize,
    idxend: usize,
) -> Block {
    let mut nvs = match parent {
        Some(p) => p.variables.clone(),
        None => vec![],
    };
    nvs.extend(args.clone());
    let mut root: Block = Block {
        children: vec![],
        variables: nvs,
    };

    let mut idx = idxstart;

    while idx < idxend {
        let mut token = &tokens[idx];
        match &token.symbol {
            Symbol::If => {
                idx += 1;
                // panic!("replace this with a next_symbol_block");
                let ifargs = parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx);
                idx += 2;

                let ifendidx =
                    next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);

                let trueblock =
                    parse_block(&tokens, &input, &funsyms, Some(&root), &args, idx, ifendidx);
                idx = ifendidx + 1;

                let mut falseblock: Option<Block> = None;

                if idx < tokens.len() {
                    match &tokens[idx].symbol {
                        Symbol::BlockStart => {
                            let elseidx = next_symbol_block(
                                &tokens,
                                &input,
                                idx,
                                Symbol::BlockStart,
                                Symbol::BlockEnd,
                            ); // not very clear code. this seems to look for the end of the else statement
                            idx += 1;
                            falseblock = Some(parse_block(
                                &tokens,
                                &input,
                                &funsyms,
                                Some(&root),
                                &args,
                                idx,
                                elseidx,
                            ));

                            idx = elseidx + 1;
                        }
                        _ => {}
                    }
                }
                root.children.push(Fragment::If {
                    condition: ifargs[0].clone(),
                    trueblock: trueblock,
                    falseblock: falseblock,
                })
            }
            // Symbol::BlockStart => {}
            // Symbol::BlockEnd => {}
            Symbol::Name(name) => match funsyms.get(name) {
                Some(fnsym) => {
                    root.children.push(Fragment::InvokeExpression(parse_fncall(
                        &tokens, &input, &funsyms, &root, &args, &mut idx, &fnsym,
                    )));
                }
                None => {
                    idx += 1;
                    token = &tokens[idx];

                    match &token.symbol {
                        Symbol::Assign => {
                            idx += 1;

                            let expression =
                                parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx);
                            root.children.push(Fragment::Assignment {
                                name: name.clone(),
                                value: expression,
                            })
                        }
                        _ => unexpected_name_exception(&input, token.index, token.symbol.clone()),
                    }
                }
            },
            _ => unexpected_symbol_exception(
                &input,
                token.index,
                Fragment::Block(root.clone()),
                token.symbol.clone(),
            ),
        }

        idx += 1;
    }
    root
}
pub fn parse_fncall(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    args: &Vec<String>,
    idx: &mut usize,
    fnsym: &FunSym,
) -> ExpressionFragment {
    let fnargs = if fnsym.args.len() != 0 {
        *idx += 1;
        parse_args(
            &tokens,
            &input,
            &funsyms,
            &parent,
            &args,
            fnsym.args.len(),
            &mut *idx,
        )
    } else {
        vec![]
    };

    // this is the one of the worst ternary implementations i've ever seen but ok
    return ExpressionFragment::Call {
        name: fnsym.name.clone(),
        args: fnargs,
    };
}

#[derive(Debug, Clone)]
pub struct Root {
    root: Block,
    functions: HashMap<String, Fragment>, // includes: Vec<String>
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
        falseblock: Option<Block>,
    },
    Function {
        name: String,
        args: Vec<String>,
        source: Block,
    },
    Assignment {
        name: String,
        value: Vec<Vec<ExpressionFragment>>,
    },
    Return(Vec<ExpressionFragment>),
    InvokeExpression(ExpressionFragment),
}
#[derive(Debug, Clone)]
pub enum ExpressionFragment {
    Name(String), //remember that this could also be a function call :/
    Op(Op),
    Logop(Logop),
    Constant(Constant),
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
pub enum Constant {
    String(String),
    Number(f64),
    // Bool(bool),
}
