//TODO : replace the clones with borrows and replace the strings with &strs

use crate::builtins;
use crate::lexer::Op;
use crate::lexer::Symbol;
use crate::lexer::Token;
use crate::libp;
use crate::libp::Library;
use core::panic;
// use
use std::collections::HashMap;
use std::env::VarError;
use std::vec;

use crate::exceptions::*;
// -> (Block, Vec<Fragment>)
pub fn parse(tkns: Vec<Token>, input: String) -> Root {
    let mut tokens = tkns.clone();
    let libs: Vec<Library> = find_loads(&mut tokens, &input)
        .iter()
        .map(|f| libp::load_lib(f.to_string()))
        .collect();

    let mut funsyms = HashMap::new();
    for lib in libs.clone() {
        for bind in lib.binds {
            funsyms.insert(
                bind.0.clone(),
                FunSym {
                    name: bind.0,
                    source: None,
                    args: bind.1.args,
                },
            );
        }
    }
    for builtin in builtins::functions() {
        funsyms.insert(
            builtin.0.clone(),
            FunSym {
                name: builtin.0,
                source: None,
                args: builtin.1.args,
            },
        );
    }
    make_funsyms(&mut tokens, &input, &mut funsyms);

    let rootblock = parse_block(&tokens, &input, &funsyms, None, &vec![], 0, tokens.len());

    let functions = make_functions(&funsyms, &input, &rootblock);
    Root {
        root: rootblock,
        functions,
        libraries: libs,
    }
}
fn make_functions(
    funsyms: &HashMap<String, FunSym>,
    input: &String,
    scope: &Block,
) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    for sym in funsyms {
        match &sym.1.source {
            Some(source) => {
                functions.insert(
                    sym.0.to_string(),
                    Function {
                        name: sym.0.to_string(),
                        args: sym.1.args.clone(),
                        source: parse_block(
                            &source,
                            input,
                            funsyms,
                            Some(scope),
                            &sym.1.args,
                            0,
                            source.len(),
                        ),
                    },
                );
            }
            None => (),
        }
    }
    functions
}
fn make_funsyms(tokens: &mut Vec<Token>, input: &String, funsyms: &mut HashMap<String, FunSym>) {
    let mut idx = 0;
    while idx < tokens.len() {
        match tokens[idx].symbol.clone() {
            Symbol::Name(funcname) => {
                match funsyms.get(&funcname) {
                    Some(sym) => {
                        //     // we want to avoid calling parse_args since it prevents functions from being called before definition. avoid using this
                        //     // i

                        //     // let mut dx = idx;
                        //     // loop{
                        //     //     match &tokens[dx]{

                        //     //     }
                        //     // }
                        //     parse_args(
                        //         &tokens,
                        //         &input,
                        //         &funsyms,
                        //         &Block {
                        //             children: vec![],
                        //             variables: vec![],
                        //         },
                        //         &vec![],
                        //         sym.args.len(),
                        //         &mut idx,
                        //     );
                        idx += 1;
                        break;
                    }
                    _ => {
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
                                    let mut tkns: Vec<Token> =
                                        tokens.drain(startidx..idx + 1).collect(); //drains the tokens. messy but i can't think of a better way of doing this
                                    tkns.pop();
                                    tkns.drain(0..args.len() + 2);
                                    idx = startidx;
                                    funsyms.insert(
                                        funcname.clone(),
                                        FunSym {
                                            name: funcname.to_string(),
                                            source: Some(tkns),
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
            Symbol::Load => {
                idx += 2;
            }
            _ => idx += 1,
        }
    }
}
fn find_loads(tokens: &mut Vec<Token>, input: &String) -> Vec<String> {
    let mut idx = 0;
    let mut loads = vec![];
    while idx < tokens.len() {
        match &tokens[idx].symbol {
            Symbol::Load => {
                idx += 1;
                let token = &tokens[idx];
                match &token.symbol {
                    Symbol::Name(libname) => {
                        loads.push(libname.clone());
                        tokens.drain(idx - 1..idx + 1);
                        idx -= 1;
                    }
                    _ => unexpected_symbol_exception(
                        &input,
                        token.index,
                        Block {
                            children: vec![],
                            variables: vec![],
                        },
                        token.symbol.clone(),
                    ),
                };
            }
            _ => idx += 1,
        }
    }
    loads
}

/// EXPECTED BEHAVIOR: Start idx just before the expression, idx will end up after the expression
fn parse_args(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    scope: &Block,
    exargs: &Vec<String>,
    argslen: usize,
    idx: &mut usize,
) -> Vec<Expression> {
    let start = idx.clone();
    let mut args: Vec<Expression> = vec![];
    // fix this its brokeen
    // keep running this code until every required argument has been captured
    while args.len() < argslen {
        let mut exp: Expression = vec![];

        // the buffer for the current argument

        let mut const_valid = true;
        loop {
            if *idx >= tokens.len() {
                break;
            }
            let token = &tokens[*idx];
            match &token.symbol {
                Symbol::Constant(_) | Symbol::Name(_) => {
                    if const_valid {
                        // if it's the first element in the array or part of an expression
                        match token.symbol.clone() {
                            Symbol::Constant(c) => {
                                exp.push(ExpressionFragment::Constant(c));
                            }
                            Symbol::Name(n) => {
                                // make sure to check if the name is valid
                                exp.push(ExpressionFragment::VarRef(parse_name(
                                    tokens, input, funsyms, scope, exargs, idx, &n,
                                )));

                                *idx -= 1;
                            }
                            _ => panic!(),
                        }
                        *idx += 1;
                        const_valid = false;
                    } else {
                        if args.len() < argslen {
                            *idx -= 1;
                            // move onto next argument
                            break;
                        } else {
                            unexpected_symbol_exception(
                                &input,
                                token.index,
                                scope.clone(),
                                token.symbol.clone(),
                            );
                        }
                    }
                }
                Symbol::Op(o) => {
                    if !const_valid {
                        exp.push(ExpressionFragment::Op(o.clone()));
                        *idx += 1;
                        const_valid = true;
                    } else {
                        unexpected_symbol_exception(
                            &input,
                            token.index,
                            scope.clone(),
                            token.symbol.clone(),
                        )
                    }
                }
                Symbol::ParenStart => {
                    *idx += 1;
                    let endidx = next_symbol_block(
                        &tokens,
                        &input,
                        *idx,
                        Symbol::ParenStart,
                        Symbol::ParenEnd,
                    );
                    exp.push(ExpressionFragment::Expression(
                        parse_args(&tokens, &input, &funsyms, &scope, &exargs, 1, idx)[0].clone(),
                    ));
                    *idx = endidx + 1;
                    const_valid = false;
                }
                _ => {
                    *idx -= 1;
                    break;
                }
            }
        }
        if exp.len() > 0 {
            args.push(exp);
        } else {
            // dbg!("breaking");
            *idx -= 1;
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
fn next_symbol(tokens: &Vec<Token>, input: &String, start: usize, end: Symbol) -> usize {
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
fn next_symbol_block(
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

fn parse_block(
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
        // dbg!(&root.children);
        match &token.symbol {
            Symbol::If => {
                idx += 1;
                // panic!("replace this with a next_symbol_block");
                let ifargs = parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx);
                idx += 1;
                let ifendidx =
                    next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);

                let trueblock =
                    parse_block(&tokens, &input, &funsyms, Some(&root), &args, idx, ifendidx);
                idx = ifendidx + 1;

                let mut falseblock: Option<Block> = None;

                if idx < tokens.len() {
                    match &tokens[idx].symbol {
                        Symbol::Else => {
                            idx += 1;
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

                                        idx = elseidx;
                                    }
                                    Symbol::If => {
                                        panic!("sorry that's complicated and i'm dumb");
                                    }
                                    _ => unexpected_symbol_exception(
                                        &input,
                                        token.index,
                                        root.clone(),
                                        token.symbol.clone(),
                                    ),
                                }
                            }
                        }
                        _ => {
                            idx -= 1;
                        }
                    }
                }
                root.children.push(Fragment {
                    frag: Frag::If {
                        condition: ifargs[0].clone(),
                        trueblock: trueblock,
                        falseblock: falseblock,
                    },
                    index: tokens[idx - 1].index,
                })
            }
            Symbol::Name(name) => {
                if !root.variables.contains(name)
                    && !args.contains(name)
                    && !funsyms.contains_key(name)
                    && match &tokens[idx + 1].symbol {
                        Symbol::Assign => true,
                        _ => false,
                    }
                {
                    // initialize variable
                    idx += 2;
                    let v =
                        parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)[0].clone();

                    idx -= 1;
                    root.variables.push(name.clone());
                    root.children.push(Fragment {
                        frag: Frag::Initialize {
                            variable: name.clone(),
                            value: v,
                        },
                        index: tokens[idx - 1].index,
                    })
                } else {
                    let startidx = idx;
                    let vref = parse_name(tokens, input, funsyms, &root, &args, &mut idx, name);

                    if if let Some(last) = vref.operations.last() {
                        match last {
                            VarRefFragment::ObjectCall { name, args } => true,
                            _ => false,
                        }
                    } else {
                        match &vref.root {
                            VarRefRoot::Call(_) => true,
                            VarRefRoot::Variable(_) => false,
                        }
                    } {
                        // it's a call
                        root.children.push(Fragment {
                            frag: Frag::Call(vref),
                            index: tokens[startidx].index,
                        });
                        idx -= 1;
                    } else {
                        // it's an assign
                        match &tokens[idx].symbol {
                            Symbol::Assign => {
                                idx += 1;
                                let v = parse_args(
                                    &tokens, &input, &funsyms, &root, &args, 1, &mut idx,
                                )[0]
                                .clone();

                                idx -= 1;
                                root.children.push(Fragment {
                                    frag: Frag::Assignment {
                                        variable: vref,
                                        value: v,
                                    },
                                    index: tokens[idx - 1].index,
                                });
                            }
                            _ => todo!(),
                        }
                    }
                }
                // root.children.push(Fragment{
                //     frag: Frag::
                //     index:tokens[idx-1].index
                // });
                //match funsyms.get(name) {
                // Some(fnsym) => {
                //     root.children.push(Fragment {
                //         frag: Frag::Call(parse_fncall(
                //             &tokens, &input, &funsyms, &root, &args, &mut idx, &fnsym,
                //         )),
                //         index: token.index,
                //     });
                // }
                // None => {
                //     // dbg!(&tokens[idx]);
                //     idx += 1;
                //     token = &tokens[idx];
                //     // dbg!(&args);
                //     match &token.symbol {
                //         Symbol::IndexStart => {
                //             idx += 1;
                //             let arg = parse_args(tokens, input, funsyms, &root, &args, 1, &mut idx)
                //                 [0]
                //             .clone();
                //             idx += 1;
                //             dbg!(&tokens[idx]);
                //             match &tokens[idx].symbol {
                //                 Symbol::Assign => {
                //                     idx += 1;
                //                     let v = parse_args(
                //                         &tokens, &input, &funsyms, &root, &args, 1, &mut idx,
                //                     )[0]
                //                     .clone();

                //                     root.children.push(Fragment {
                //                         frag: Frag::IndexAssignment {
                //                             name: name.clone(),
                //                             index: arg,
                //                             value: v,
                //                         },
                //                         index: token.index,
                //                     });

                //                     idx -= 1;
                //                 }
                //                 _ => panic!("make this exception later"),
                //             }
                //         }
                //         Symbol::Assign => {
                //             idx += 1;
                //             // dbg!(
                //             //     &parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)
                //             //         [0]
                //             // );
                //             let v =
                //                 parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)[0]
                //                     .clone();
                //             root.variables.push(name.to_string());
                //             root.children.push(Fragment {
                //                 frag: Frag::Assignment {
                //                     name: name.clone(),
                //                     value: v, // potentially unsafe code whatever
                //                 },
                //                 index: token.index,
                //             });
                //             idx -= 1;
                //             // dbg!(&tokens[idx]);
                //             // idx += 2;
                //         }
                //         _ => unexpected_name_exception(
                //             &input,
                //             tokens[idx - 1].index,
                //             tokens[idx - 1].symbol.clone(),
                //         ),
                //     }
                // }
            }
            Symbol::BlockStart => {
                let endidx =
                    next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);
                idx += 1;
                root.children.push(Fragment {
                    frag: Frag::Block(parse_block(
                        &tokens,
                        &input,
                        &funsyms,
                        Some(&root),
                        &args,
                        idx,
                        endidx,
                    )),
                    index: token.index,
                });
                idx = endidx;
            }
            Symbol::Loop => {
                idx += 1;
                let endidx =
                    next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);
                idx += 1;
                root.children.push(Fragment {
                    frag: Frag::Loop(parse_block(
                        &tokens,
                        &input,
                        &funsyms,
                        Some(&root),
                        &args,
                        idx,
                        endidx,
                    )),
                    index: token.index,
                });
                idx = endidx;
            }
            Symbol::Break => root.children.push(Fragment {
                frag: Frag::Break,
                index: token.index,
            }),
            Symbol::Return => {
                idx += 1;
                root.children.push(Fragment {
                    frag: Frag::Return(
                        parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)[0].clone(),
                    ),
                    index: token.index,
                });
            }
            Symbol::For => {
                idx += 1;
                let name = match &tokens[idx].symbol {
                    Symbol::Name(n) => n.clone(),
                    _ => panic!(),
                };
                dbg!(&name);
                idx += 1;
                let mut innerargs = args.clone();
                innerargs.push(name.clone());
                let forargs = parse_args(&tokens, &input, &funsyms, &root, &innerargs, 2, &mut idx);
                //parse block here
                let startidx = next_symbol(&tokens, &input, idx, Symbol::BlockStart);

                let incrementorblock = parse_block(
                    &tokens,
                    &input,
                    &funsyms,
                    Some(&root),
                    &innerargs,
                    idx,
                    startidx,
                );
                idx = startidx + 1;
                let endidx =
                    next_symbol_block(&tokens, &input, idx, Symbol::BlockStart, Symbol::BlockEnd);

                let block = parse_block(
                    &tokens,
                    &input,
                    &funsyms,
                    Some(&root),
                    &innerargs,
                    idx,
                    endidx,
                );
                idx = endidx + 1;

                root.children.push(Fragment {
                    frag: Frag::For {
                        name: name,
                        initial: forargs[0].clone(),
                        condition: forargs[1].clone(),
                        incrementor: incrementorblock,
                        block,
                    },
                    index: tokens[idx - 1].index,
                });
                idx -= 1;
            }

            _ => {
                unexpected_symbol_exception(&input, token.index, root.clone(), token.symbol.clone())
            }
        }

        idx += 1;
    }
    root
}

fn parse_name(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    scope: &Block,
    exargs: &Vec<String>,
    idx: &mut usize,
    name: &String,
) -> VarRef {
    let mut fragments = vec![];

    let root = if scope.variables.contains(name) || exargs.contains(name) {
        VarRefRoot::Variable(name.clone())
    } else if funsyms.contains_key(name) {
        let fnsym = funsyms.get(name).unwrap();
        VarRefRoot::Call(parse_fncall(
            tokens, input, funsyms, scope, exargs, idx, &fnsym,
        ))
    } else {
        unexpected_name_exception(&input, tokens[*idx].index, Symbol::Name(name.clone()));
        panic!("rust moment");
    };

    *idx += 1;
    while *idx < tokens.len() {
        match &tokens[*idx].symbol {
            Symbol::IndexObject => {
                *idx += 1;
                let funcname = match &tokens[*idx].symbol {
                    Symbol::Name(n) => n,
                    _ => {
                        unexpected_symbol_exception(
                            input,
                            *idx,
                            scope.clone(),
                            tokens[*idx].symbol.clone(),
                        );
                        panic!("this is unreachable but the compiler doesn't know that so :/ ");
                    }
                };
                *idx += 1;
                match &tokens[*idx].symbol {
                    Symbol::ParenStart => {
                        *idx += 1;
                        let endidx = match &tokens[*idx].symbol {
                            Symbol::ParenEnd => *idx,
                            _ => next_symbol_block(
                                tokens,
                                input,
                                *idx,
                                Symbol::ParenStart,
                                Symbol::ParenEnd,
                            ),
                        };

                        let mut args: Vec<Expression> = vec![];

                        while *idx < endidx {
                            args.append(&mut parse_args(
                                tokens, input, funsyms, scope, exargs, 1, idx,
                            ));
                        }
                        fragments.push(VarRefFragment::ObjectCall {
                            name: funcname.clone(),
                            args,
                        });
                    }
                    _=>{
                        
                    }
                }
            }
            Symbol::IndexStart => {
                *idx += 1;
                let arg = parse_args(tokens, input, funsyms, scope, exargs, 1, idx)[0].clone();
                fragments.push(VarRefFragment::IndexInto(arg));
                // *idx += 1;
            }

            _ => break,
        }
        *idx += 1;
    }
    //     Symbol::IndexStart => {
    //         *idx += 2;
    //         let arg = parse_args(
    //             tokens, input, funsyms, scope, exargs, 1, idx,
    //         )[0]
    //         .clone();

    //         exp.push(ExpressionFragment::IndexName {
    //             name: n.clone(),
    //             index: arg,
    //         });

    //         true
    //     }
    //     Symbol::IndexObject => {
    //         *idx += 2;
    //         let funcname = match &tokens[*idx].symbol {
    //             Symbol::Name(n) => n,
    //             _ => {
    //                 unexpected_symbol_exception(
    //                     input,
    //                     *idx,
    //                     scope.clone(),
    //                     tokens[*idx].symbol.clone(),
    //                 );
    //                 panic!("this is unreachable but the compiler doesn't know that so :/ ");
    //             }
    //         };
    //         *idx += 2;
    //         let endidx = match &tokens[*idx].symbol {
    //             Symbol::ParenEnd => *idx,
    //             _ => next_symbol_block(
    //                 tokens,
    //                 input,
    //                 *idx,
    //                 Symbol::ParenStart,
    //                 Symbol::ParenEnd,
    //             ),
    //         };

    //         let mut args: Vec<Expression> = vec![];

    //         while *idx < endidx {
    //             args.append(&mut parse_args(
    //                 tokens, input, funsyms, scope, exargs, 1,
    //                 idx,
    //             ));
    //         }
    //         exp.push(ExpressionFragment::ObjectCall {
    //             objectname: n.clone(),
    //             functionname: funcname.clone(),
    //             args,
    //         });
    //         // also we need to have dlls to be able to add more types.

    //         true
    //     }
    //     _ => false,
    // }
    VarRef {
        root: root,
        operations: fragments,
    }
}
fn parse_fncall(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    args: &Vec<String>,
    idx: &mut usize,
    fnsym: &FunSym,
) -> Call {
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
    if fnsym.args.len() > 0 {
        *idx -= 1;
    }
    return Call {
        name: fnsym.name.clone(),
        args: fnargs,
    };
}

#[derive(Debug, Clone)]
pub struct Root {
    pub root: Block,
    pub functions: HashMap<String, Function>, // includes: Vec<String>
    pub libraries: Vec<Library>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub children: Vec<Fragment>,
    pub variables: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct FunSym {
    name: String,
    source: Option<Vec<Token>>,
    args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub frag: Frag,
    pub index: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Frag {
    If {
        condition: Expression,
        trueblock: Block,
        falseblock: Option<Block>,
    },
    For {
        name: String,
        initial: Expression,
        condition: Expression,
        incrementor: Block,
        block: Block,
    },
    Call(VarRef),
    Block(Block),
    Loop(Block),
    Break,
    Initialize {
        variable: String,
        value: Expression,
    },
    Assignment {
        variable: VarRef,
        value: Expression,
    },
    Return(Expression),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub source: Block,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionFragment {
    Op(Op),
    Constant(Constant),
    Call(Call),
    Expression(Expression),
    VarRef(VarRef),
    // Name(String), //remember that this could also be a function call :/
    // IndexName {
    //     name: String,
    //     index: Expression,
    // },
    // ObjectCall {
    //     objectname: String,
    //     functionname: String,
    //     args: Vec<Expression>,
    // },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarRef {
    pub root: VarRefRoot,
    pub operations: Vec<VarRefFragment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarRefRoot {
    Variable(String),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarRefFragment {
    IndexInto(Expression),
    ObjectProperty(String),
    ObjectCall { name: String, args: Vec<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    String(String),
    Number(i64),
    Bool(bool),
}

type Expression = Vec<ExpressionFragment>;
