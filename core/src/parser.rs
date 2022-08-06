//TODO : replace the clones with borrows and replace the strings with &strs

use crate::builtins;
use crate::lexer::Op;
use crate::lexer::Symbol;
use crate::lexer::Token;
use crate::runtime::RFunction;
use core::panic;
use std::collections::HashMap;

use crate::exceptions::*;
pub fn parse(
    tkns: Vec<Token>,
    input: &String,
    funcs: &HashMap<String, RFunction>,
) -> Result<Root, Exception> {
    let mut tokens = tkns;
    let mut funsyms = HashMap::new();
    for f in funcs {
        funsyms.insert(
            f.0.clone(),
            FunSym {
                name: f.0.clone(),
                source: None,
                args: f.1.args.clone(),
            },
        );
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
    make_funsyms(&mut tokens, &mut funsyms)?;

    let rootblock = parse_block(&tokens, input, &funsyms, None, &vec![], 0, tokens.len())?;

    let functions = make_functions(&funsyms, input, &rootblock)?;
    Ok(Root {
        root: rootblock,
        functions,
    })
}
fn make_functions(
    funsyms: &HashMap<String, FunSym>,
    input: &String,
    scope: &Block,
) -> Result<HashMap<String, Function>, Exception> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    for sym in funsyms {
        match &sym.1.source {
            Some(source) => {
                functions.insert(
                    sym.0.to_string(),
                    Function {
                        args: sym.1.args.clone(),
                        source: parse_block(
                            source,
                            input,
                            funsyms,
                            Some(scope),
                            &sym.1.args,
                            0,
                            source.len(),
                        )?,
                    },
                );
            }
            None => (),
        }
    }
    Ok(functions)
}
fn make_funsyms(
    tokens: &mut Vec<Token>,
    funsyms: &mut HashMap<String, FunSym>,
) -> Result<(), Exception> {
    let mut idx = 0;
    while idx < tokens.len() {
        match tokens[idx].symbol.clone() {
            Symbol::Name(funcname) => {
                match funsyms.get(&funcname) {
                    Some(_) => {
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
                                        tokens,
                                        idx,
                                        Symbol::BlockStart,
                                        Symbol::BlockEnd,
                                    )?;
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
                                            args,
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
                let endx = next_symbol(tokens, idx, Symbol::BlockEnd)?;
                //WARNGIN : WILL ACT WEIRDLY ON NESTED IF BLOCKS. SHOULDN"T MATTER
                idx = endx;
            }
            Symbol::Load => {
                idx += 2;
            }
            _ => idx += 1,
        }
    }
    Ok(())
}
pub fn find_loads(tokens: &mut Vec<Token>) -> Result<Vec<String>, Exception> {
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
                    _ => {
                        return Err(Exception::unexpected_symbol(
                            token.index,
                            token.symbol.clone(),
                            vec![Symbol::Name("a".into())],
                        ))
                    }
                };
            }
            _ => idx += 1,
        }
    }
    Ok(loads)
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
) -> Result<Vec<Expression>, Exception> {
    let start = *idx;
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
                Symbol::Constant(_) | Symbol::Name(_) | Symbol::Lambda => {
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
                                )?));

                                *idx -= 1;
                            }
                            Symbol::Lambda => {
                                // implement capturing, do later
                                *idx += 1;
                                let mut args = vec![];
                                loop {
                                    match &tokens[*idx].symbol {
                                        Symbol::Name(s) => args.push(s.clone()),
                                        Symbol::Lambda => break,
                                        _ => {
                                            return Err(Exception::unexpected_symbol(
                                                tokens[*idx].index,
                                                tokens[*idx].symbol.clone(), // later (remove excessive cloning)
                                                vec![Symbol::Lambda, Symbol::Name("".into())],
                                            ));
                                        }
                                    }
                                    *idx += 1;
                                }
                                *idx += 1;
                                let endidx = next_symbol_block(
                                    tokens,
                                    *idx,
                                    Symbol::BlockStart,
                                    Symbol::BlockEnd,
                                )?;
                                let block =
                                    parse_block(tokens, input, funsyms, None, &args, *idx, endidx)?;
                                exp.push(ExpressionFragment::Lambda(Function {
                                    args,
                                    source: block,
                                }));
                                *idx = endidx;
                            }
                            _ => {
                                return Err(Exception::new(
                                    tokens[*idx].index,
                                    "InvalidExpressionException",
                                    "This expression is not valid",
                                ))
                            }
                        }
                        *idx += 1;
                        const_valid = false;
                    } else if args.len() < argslen {
                        *idx -= 1;
                        // move onto next argument
                        break;
                    } else {
                        return Err(Exception::unexpected_symbol(
                            token.index,
                            token.symbol.clone(),
                            vec![Symbol::Lambda, Symbol::Name("".into())],
                        ));
                    }
                }
                Symbol::Op(o) => {
                    if !const_valid {
                        exp.push(ExpressionFragment::Op(o.clone()));
                        *idx += 1;
                        const_valid = true;
                    } else {
                        return Err(Exception::unexpected_symbol(
                            token.index,
                            token.symbol.clone(),
                            vec![],
                        ));
                    }
                }
                Symbol::ParenStart => {
                    *idx += 1;
                    let endidx =
                        next_symbol_block(tokens, *idx, Symbol::ParenStart, Symbol::ParenEnd)?;
                    exp.push(ExpressionFragment::Expression(
                        parse_args(tokens, input, funsyms, scope, exargs, 1, idx)?[0].clone(),
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
        if !exp.is_empty() {
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
        return Err(Exception::new(
            tokens[start - 1].index,
            "ArgumentException",
            "Not enough arguments!",
        ));
    }
    Ok(args)
}

// panic!(split the arguments parser into a separate function);

// returns the index of the next symbol
fn next_symbol(tokens: &[Token], start: usize, end: Symbol) -> Result<usize, Exception> {
    let mut idx = start;
    loop {
        let token = &tokens[idx];
        if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&end) {
            return Ok(idx);
        }
        idx += 1;
        if idx == tokens.len() {
            return Err(Exception::new(
                idx,
                "EOFexcpetion",
                &format!("Expected to find {:?}, got EOF instead", &end),
            ));
        }
    }
}
fn next_symbol_block(
    tokens: &[Token],
    start: usize,
    addepth: Symbol,
    backdepth: Symbol,
) -> Result<usize, Exception> {
    let mut idx = start;
    let mut depth = 1;
    loop {
        idx += 1;
        if idx == tokens.len() {
            return Err(Exception::new(
                idx,
                "EOFexcpetion",
                &format!("Expected to find {:?}, got EOF instead", &backdepth),
            ));
        }
        let token = &tokens[idx];
        if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&addepth) {
            depth += 1;
        } else if std::mem::discriminant(&token.symbol) == std::mem::discriminant(&backdepth) {
            depth -= 1;
            if depth == 0 {
                return Ok(idx);
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
) -> Result<Block, Exception> {
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
        let token = &tokens[idx];
        // dbg!(&root.children);
        match &token.symbol {
            Symbol::If => {
                idx += 1;
                // panic!("replace this with a next_symbol_block");
                let ifargs = parse_args(tokens, input, funsyms, &root, args, 1, &mut idx);
                idx += 1;
                let ifendidx =
                    next_symbol_block(tokens, idx, Symbol::BlockStart, Symbol::BlockEnd)?;

                let trueblock =
                    parse_block(tokens, input, funsyms, Some(&root), args, idx, ifendidx)?;
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
                                            tokens,
                                            idx,
                                            Symbol::BlockStart,
                                            Symbol::BlockEnd,
                                        )?;
                                        idx += 1;
                                        falseblock = Some(parse_block(
                                            tokens,
                                            input,
                                            funsyms,
                                            Some(&root),
                                            args,
                                            idx,
                                            elseidx,
                                        )?);

                                        idx = elseidx;
                                    }
                                    Symbol::If => {
                                        panic!("sorry that's complicated and i'm dumb");
                                    }
                                    _ => {
                                        return Err(Exception::unexpected_symbol(
                                            token.index,
                                            token.symbol.clone(),
                                            vec![Symbol::BlockStart, Symbol::If],
                                        ))
                                    }
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
                        condition: ifargs?[0].clone(),
                        trueblock,
                        falseblock,
                    },
                    index: tokens[idx - 1].index,
                })
            }
            Symbol::Name(name) => {
                if !root.variables.contains(name)
                    && !args.contains(name)
                    && !funsyms.contains_key(name)
                    && matches!(&tokens[idx + 1].symbol, Symbol::Assign)
                {
                    // initialize variable
                    idx += 2;
                    let v = parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)?[0]
                        .clone();

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
                    let vref = parse_name(tokens, input, funsyms, &root, &args, &mut idx, name)?;

                    if if let Some(last) = vref.operations.last() {
                        match last {
                            VarRefFragment::ObjectCall { name: _, args: _ } => true,
                            VarRefFragment::LambdaCall(_) => true,
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
                        match &tokens.sget(idx, vec![Symbol::Assign])? {
                            Symbol::Assign => {
                                idx += 1;
                                let v = parse_args(
                                    &tokens, &input, &funsyms, &root, &args, 1, &mut idx,
                                )?[0]
                                    .clone();

                                idx -= 1;
                                root.children.push(Fragment {
                                    frag: Frag::Assignment {
                                        variable: vref,
                                        value: v,
                                    },
                                    index: tokens[idx - 2].index,
                                });
                            }
                            _ => {
                                return Err(Exception::unexpected_symbol(
                                    idx,
                                    Symbol::Assign,
                                    vec![],
                                ))
                            }
                        }
                    }
                }
            }
            Symbol::BlockStart => {
                let endidx = next_symbol_block(&tokens, idx, Symbol::BlockStart, Symbol::BlockEnd)?;
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
                    )?),
                    index: token.index,
                });
                idx = endidx;
            }
            Symbol::Loop => {
                idx += 1;
                let endidx = next_symbol_block(&tokens, idx, Symbol::BlockStart, Symbol::BlockEnd)?;
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
                    )?),
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
                        parse_args(&tokens, &input, &funsyms, &root, &args, 1, &mut idx)?[0]
                            .clone(),
                    ),
                    index: token.index,
                });
            }
            Symbol::For => {
                idx += 1;
                let tkn = &tokens.sget(idx, vec![Symbol::Name(String::new())])?;
                let name = if let Symbol::Name(n) = tkn {
                    n.clone()
                } else {
                    return Err(Exception::unexpected_symbol(
                        idx,
                        tkn.clone().clone(),
                        vec![Symbol::Name(String::default())],
                    ));
                };
                idx += 1;
                let mut innerargs = args.clone();
                innerargs.push(name.clone());
                let forargs = parse_args(tokens, input, funsyms, &root, &innerargs, 2, &mut idx)?;
                //parse block here
                let startidx = next_symbol(tokens, idx, Symbol::BlockStart)?;

                let incrementorblock = parse_block(
                    tokens,
                    input,
                    funsyms,
                    Some(&root),
                    &innerargs,
                    idx,
                    startidx,
                )?;
                idx = startidx + 1;
                let endidx = next_symbol_block(tokens, idx, Symbol::BlockStart, Symbol::BlockEnd)?;

                let block =
                    parse_block(tokens, input, funsyms, Some(&root), &innerargs, idx, endidx)?;
                idx = endidx + 1;

                root.children.push(Fragment {
                    frag: Frag::For {
                        name,
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
                return Err(Exception::unexpected_symbol(
                    token.index,
                    token.symbol.clone(),
                    vec![],
                ))
            }
        }

        idx += 1;
    }
    Ok(root)
}

fn parse_name(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    scope: &Block,
    exargs: &Vec<String>,
    idx: &mut usize,
    name: &String,
) -> Result<VarRef, Exception> {
    let mut fragments = vec![];

    let root = if scope.variables.contains(name) || exargs.contains(name) {
        VarRefRoot::Variable(name.clone())
    } else if funsyms.contains_key(name) {
        let fnsym = funsyms.get(name).unwrap();
        VarRefRoot::Call(parse_fncall(
            tokens, input, funsyms, scope, exargs, idx, fnsym,
        )?)
    } else {
        return Err(Exception::unexpected_name(
            tokens[*idx].index,
            Symbol::Name(name.clone()),
        ));
    };

    *idx += 1;
    while *idx < tokens.len() {
        match &tokens[*idx].symbol {
            Symbol::IndexObject => {
                *idx += 1;
                let funcname = match &tokens[*idx].symbol {
                    Symbol::Name(n) => n,
                    _ => {
                        return Err(Exception::unexpected_symbol(
                            *idx,
                            tokens[*idx].symbol.clone(),
                            vec![Symbol::Name("".into())],
                        ));
                    }
                };
                *idx += 1;
                match tokens.get(*idx) {
                    Some(tkn) => match tkn.symbol {
                        Symbol::ParenStart => {
                            *idx += 1;

                            fragments.push(VarRefFragment::ObjectCall {
                                name: funcname.clone(),
                                args: parse_args_with_parens(
                                    tokens, input, funsyms, scope, exargs, idx,
                                )?,
                            });
                        }
                        _ => {
                            *idx -= 1;
                            fragments.push(VarRefFragment::ObjectProperty(funcname.clone()))
                        }
                    },
                    None => {
                        *idx -= 1;
                        fragments.push(VarRefFragment::ObjectProperty(funcname.clone()))
                    }
                }
            }
            Symbol::IndexStart => {
                *idx += 1;
                let arg = parse_args(tokens, input, funsyms, scope, exargs, 1, idx)?[0].clone();
                fragments.push(VarRefFragment::IndexInto(arg));
            }
            Symbol::ParenStart => {
                *idx += 1;

                fragments.push(VarRefFragment::LambdaCall(parse_args_with_parens(
                    tokens, input, funsyms, scope, exargs, idx,
                )?));
            }

            _ => break,
        }
        *idx += 1;
    }
    Ok(VarRef {
        root,
        operations: fragments,
    })
}
fn parse_args_with_parens(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    scope: &Block,
    exargs: &Vec<String>,
    idx: &mut usize,
) -> Result<Vec<Expression>, Exception> {
    let endidx = match &tokens[*idx].symbol {
        Symbol::ParenEnd => *idx,
        _ => next_symbol_block(tokens, *idx, Symbol::ParenStart, Symbol::ParenEnd)?,
    };

    let mut args: Vec<Expression> = vec![];

    while *idx < endidx {
        args.append(&mut parse_args(
            tokens, input, funsyms, scope, exargs, 1, idx,
        )?);
    }

    let v = "A";

    Ok(args)
}
fn parse_fncall(
    tokens: &Vec<Token>,
    input: &String,
    funsyms: &HashMap<String, FunSym>,
    parent: &Block,
    args: &Vec<String>,
    idx: &mut usize,
    fnsym: &FunSym,
) -> Result<Call, Exception> {
    let fnargs = if !fnsym.args.is_empty() {
        *idx += 1;
        parse_args(
            tokens,
            input,
            funsyms,
            parent,
            args,
            fnsym.args.len(),
            &mut *idx,
        )?
    } else {
        vec![]
    };
    if !fnsym.args.is_empty() {
        *idx -= 1;
    }
    Ok(Call {
        name: fnsym.name.clone(),
        args: fnargs,
    })
}
impl CatchGet for Vec<Token> {
    fn sget(&self, idx: usize, allowed: Vec<Symbol>) -> Result<&Symbol, Exception> {
        if let Some(t) = self.get(idx) {
            Ok(&t.symbol)
        } else {
            // lazy. fix later
            Err(Exception::unexpected_symbol(
                idx,
                Symbol::Name("EndOfFile".into()),
                allowed,
            ))
        }
    }
}
trait CatchGet {
    fn sget(&self, getidx: usize, allowed: Vec<Symbol>) -> Result<&Symbol, Exception>;
}
// fn safeget()

#[derive(Debug, Clone)]
pub struct Root {
    pub root: Block,
    pub functions: HashMap<String, Function>, // includes: Vec<String>
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
pub struct ConditionalBlock {
    block: Block,
    condition: Expression,
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
    Lambda(Function),
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
    LambdaCall(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    String(String),
    Number(f32),
    Bool(bool),
}

type Expression = Vec<ExpressionFragment>;
