//TODO : replace the clones with borrows and replace the strings with &strs
// Split the argparser into a function since it's called several times

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
    let mut funsyms = make_funsyms(&mut tokens);

    // //dbg!(Token::InternalCall {});

    funsyms.insert(
        String::from("print"),
        FunSym {
            name: String::from("print"),
            args: vec![String::from("text")],
            source: vec![],
        },
    );

    //dbg!(&funsyms);
    let functions = make_functions(&funsyms, &input, &root);

    //dbg!(functions);

    //dbg!("??");
    //dbg!(&tokens);

    dbg!(parse_block(
        &tokens,
        &input,
        &funsyms,
        &root,
        &vec![],
        0,
        tokens.len()
    )); // substitute vec![] for global constants later

    // (blockparse(tokens, input), functions)
}
pub fn make_functions(
    funsyms: &HashMap<String, FunSym>,
    input: &String,
    parent: &Block,
) -> HashMap<String, Fragment> {
    let mut functions: HashMap<String, Fragment> = HashMap::new();
    for sym in funsyms {
        dbg!(&sym.1.args);
        functions.insert(
            sym.0.to_string(),
            Fragment::Function {
                name: sym.0.to_string(),
                args: sym.1.args.clone(),
                source: parse_block(
                    &sym.1.source,
                    input,
                    funsyms,
                    parent,
                    &sym.1.args,
                    0,
                    sym.1.source.len(),
                ),
            },
        );
    }
    functions
}
pub fn make_funsyms(tokens: &mut Vec<Token>) -> HashMap<String, FunSym> {
    let mut idx = 0;
    let mut funsyms: HashMap<String, FunSym> = HashMap::new();
    while idx < tokens.len() {
        //dbg!("looping");
        match tokens[idx].symbol.clone() {
            Symbol::Name(funcname) => {
                let startidx = idx;
                let mut args: Vec<String> = vec![];
                loop {
                    //dbg!("looping");
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
                            //dbg!(&tokens);
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

pub fn parse_args(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    exargs: &Vec<String>,
    argslen: usize,
    start: usize,
    end: usize,
) -> Vec<Vec<ExpressionFragment>> {
    let mut args: Vec<Vec<ExpressionFragment>> = vec![];

    let mut idx = start;
    // keep running this code until every required argument has been captured
    while args.len() != argslen {
        let mut exp: Vec<ExpressionFragment> = vec![];

        // //dbg!(tokens);

        // the buffer for the current argument
        loop {
            idx += 1;
            dbg!(idx);
            dbg!(end);
            if idx > end {
                break;
            }
            let token = &tokens[idx];
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
                                exp.push(ExpressionFragment::Value(Value::String(s)))
                            }
                            Symbol::Number(n) => {
                                exp.push(ExpressionFragment::Value(Value::Number(n)))
                            }
                            Symbol::Name(n) => {
                                // make sure to check if the name is valid
                                dbg!(&parent.variables);
                                if parent.variables.contains(&n) {
                                    exp.push(ExpressionFragment::Name(n));
                                } else if funsyms.contains_key(&n) {
                                    let fnsym = funsyms.get(&n).unwrap();
                                    exp.push(parse_fncall(
                                        &tokens, &input, &funsyms, &parent, &exargs, &mut idx,
                                        &fnsym,
                                    ));
                                } else {
                                    unexpected_name_exception(&input, token.index, Symbol::Name(n))
                                }
                            }
                            _ => panic!(),
                        }
                    } else {
                        // otherwise jump to the next argument
                        if args.len() < argslen {
                            //dbg!("jumping to next arg");
                            idx -= 1;
                            break;
                        } else {
                            //dbg!("unx in p args");
                            unexpected_symbol_exception(
                                &input,
                                token.index,
                                Fragment::Block(parent.clone()),
                                token.symbol.clone(),
                            );
                        }
                    }
                }
                Symbol::Logop(_) | Symbol::Op(_) => {}
                _ => unexpected_symbol_exception(
                    &input,
                    token.index,
                    Fragment::Block(parent.clone()),
                    token.symbol.clone(),
                ),
            }
        }
        if exp.len() > 0 {
            args.push(exp);
        } else {
            break;
        }
        // why was this commented out
    }
    if args.len() < argslen {
        exception(
            &input,
            tokens[start].index,
            "ArgumentException",
            "Not enough arguments!",
        );
    } else if idx - 1 != end {
        exception(
            &input,
            tokens[start].index,
            "ArgumentException",
            "Too many arguments!",
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
            // exception(
            //     &input,
            //     idx,
            //     "EOFexcpetion",
            //     format!("Expected to find {:?}, got EOF instead", &end),
            // );
            panic!();
        }
    }
}

pub fn parse_block(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    args: &Vec<String>,
    idxstart: usize,
    idxend: usize,
) -> Block {
    //dbg!("parsing");
    //dbg!(&tokens);

    let mut nvs = parent.variables.clone();
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
                let endidx = next_symbol(&tokens, &input, idx, Symbol::BlockStart);
                let ifargs = parse_args(
                    &tokens,
                    &input,
                    &funsyms,
                    &parent,
                    &args,
                    1,
                    idx,
                    endidx - 1,
                );
                dbg!(&ifargs);
                idx = endidx + 1;

                let ifendidx = next_symbol(&tokens, &input, idx, Symbol::BlockEnd);

                let trueblock = parse_block(&tokens, &input, &funsyms, &root, &args, idx, ifendidx);
                idx = ifendidx + 1;

                let mut falseblock: Option<Block> = None;

                match &tokens[idx].symbol {
                    Symbol::BlockStart => {
                        let elseidx = next_symbol(&tokens, &input, idx, Symbol::BlockEnd);

                        falseblock = Some(parse_block(
                            &tokens, &input, &funsyms, &root, &args, idx, elseidx,
                        ));
                    }
                    _ => {}
                }
                root.children.push(Fragment::If {
                    condition: ifargs[0].clone(),
                    trueblock: trueblock,
                    falseblock: falseblock,
                })
            }
            Symbol::Assign => {}
            // Symbol::BlockStart => {}
            // Symbol::BlockEnd => {}
            Symbol::Name(name) => match funsyms.get(name) {
                Some(fnsym) => {
                    //dbg!(&tokens);
                    //dbg!("uhaiudhasui");

                    root.children.push(Fragment::InvokeExpression(parse_fncall(
                        &tokens, &input, &funsyms, &root, &args, &mut idx, &fnsym,
                    )));
                }
                None => unexpected_name_exception(&input, token.index, token.symbol.clone()),
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
    *idx += 1;
    //dbg!(*idx);
    let mut token = &tokens[*idx];
    //dbg!(token);
    match &token.symbol {
        Symbol::ParenStart => {
            // look for end of paren, parse on that
            let endidx = next_symbol(&tokens, &input, *idx, Symbol::ParenEnd);

            dbg!(&idx);
            dbg!(endidx);
            //dbg!();
            let fnargs = parse_args(
                &tokens,
                &input,
                &funsyms,
                &parent,
                &args,
                fnsym.args.len(),
                *idx,
                endidx - 1,
            );
            dbg!(&fnargs);
            *idx = endidx;
            return ExpressionFragment::Call {
                name: fnsym.name.clone(),
                args: fnargs,
            };
        }
        _ => {
            exception(
                &input,
                token.index,
                "SyntaxException",
                "Incorrect syntax for a function call. Please use parenthesis",
            );
            panic!("cannot continue execution");
        }
    }
    // idx -= 1;
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
        value: Value,
    },
    InvokeExpression(ExpressionFragment),
}
#[derive(Debug, Clone)]
pub enum ExpressionFragment {
    Name(String), //remember that this could also be a function call :/
    Op(Op),
    Logop(Logop),
    Value(Value),
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
pub enum Value {
    String(String),
    Number(f64),
    // Bool(bool),
}