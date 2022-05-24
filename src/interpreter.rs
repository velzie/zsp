use std::{cell::RefCell, collections::HashMap, f32::consts::PI, fs, ops::DerefMut, rc::Rc};

use crate::parser::{self, Block, Constant, ExpressionFragment, Fragment, Function, Root};
use typed_arena::Arena;

// inp structure ideas:
//1: simplest possible solutions
// single function for parsing a scope, doesn't return but instead mutates a scope, when a return is called

// or

// returns an enum, none for no return and if there is a return keep going back on the stack until a function block is hit

//2: undefined behavior:
// self referential struct with parent owning child, store return value within struct and iterate from there

//3. pointers

// run a loop with a linked list of pointer to the block, shift back one pointer when the block is exited, retain

pub fn interpret(root: Root) {
    // dbg!(root);
    // root.root.Run();
    // insert environemnt variables

    let mut functions = HashMap::new();

    // load dlls

    // unsafe {
    let heap = Arena::new();
    // yay entire program is unsafe wooo
    for lib in root.libraries {
        // for root
        unsafe {
            let libobj = heap.alloc(libloading::Library::new(lib.static_loc).unwrap()); // lazy bad unsafe garbage code

            for bind in lib.binds {
                let fnc: libloading::Symbol<unsafe extern "C" fn(Vec<Value>) -> Value> =
                    libobj.get(bind.1.bound_symbol.as_bytes()).unwrap();
                functions.insert(
                    bind.0.clone(),
                    RFunction {
                        name: bind.0.clone(),
                        args: bind.1.args,
                        func: FunctionType::ExternalFunction(fnc),
                    },
                );
            }
        }
    }

    for fun in root.functions {
        let cfn = fun.1.clone();
        functions.insert(
            fun.0,
            RFunction {
                name: cfn.name.clone(),
                args: cfn.args.clone(),
                func: FunctionType::InternalFunction(cfn.clone()),
            },
        );
    }
    let mut stack: Vec<Scope> = vec![];
    stack.push(root.root.to_scope(ScopeType::Block, HashMap::new()));
    'stack: while stack.len() > 0 {
        let mut pointer = stack.last_mut().unwrap();
        while pointer.idx < pointer.structure.children.len() {
            let frag = &pointer.structure.children[pointer.idx];

            match frag {
                Fragment::If {
                    condition,
                    trueblock,
                    falseblock,
                } => {
                    if evaluate_expression(condition).to_bool() {
                        let tscope = trueblock.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(tscope);
                        continue 'stack;
                    } else if let Some(fb) = falseblock {
                        let fscope = fb.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(fscope);
                        continue 'stack;
                    }
                }
                Fragment::Call(call) => {
                    let rf = functions.get(&call.name).unwrap();
                    let args: Vec<Value> =
                        call.args.iter().map(|f| evaluate_expression(f)).collect();

                    match &rf.func {
                        FunctionType::ExternalFunction(extfnc) => unsafe {
                            extfnc(args);
                        },
                        FunctionType::InternalFunction(func) => {
                            let mut passedargs = pointer.variables.clone();
                            for (i, argname) in func.args.iter().enumerate() {
                                passedargs.insert(argname.clone(), RefCell::new(args[i].clone()));
                            }
                            let nsc = func.source.to_scope(ScopeType::Function, passedargs);
                            stack.push(nsc);
                            continue 'stack;
                        }
                    }

                    // evaluate_expression(cal);
                }
                Fragment::Assignment { name, value } => match pointer.variables.get_mut(name) {
                    Some(rc) => {
                        *rc.borrow_mut() = evaluate_expression(value);
                        // this feels very wrong
                        // buuuut it compiles
                    }
                    None => {
                        pointer
                            .variables
                            .insert(name.clone(), RefCell::new(evaluate_expression(value)));
                    }
                },
                Fragment::Block(block) => {
                    let scope = block.to_scope(ScopeType::Block, pointer.variables.clone());
                    stack.push(scope);
                    continue 'stack;
                }
                _ => {}
            }

            continue 'stack;
        }
        // match fragment {
        //     // Fragment::Break=>{
        //     //     match  {

        //     //     }
        //     // }
        //     Fragment::If {
        //         trueblock,
        //         falseblock,
        //         condition,
        //     } => {
        //         if evaluate_expression(&condition).to_bool() {
        //             trueblock.to_scope(HashMap::new()).execute(&functions);
        //         } else {
        //             match falseblock {
        //                 Some(b) => b.to_scope(HashMap::new()).execute(&functions),
        //                 None => Value::Null,
        //             };
        //         }
        //     }
        //     Fragment::Assignment { name, value } => {
        //         self.variables
        //             .insert(name.clone(), Rc::new(evaluate_expression(value)));
        //     }
        //     Fragment::InvokeExpression(exp) => {
        //         match &exp {
        //             ExpressionFragment::Call { name, args } => match functions.get(name) {
        //                 Some(func) => {
        //                     func.call(
        //                         args.iter().map(|f| evaluate_expression(f)).collect(),
        //                         functions,
        //                     );
        //                 }
        //                 None => panic!(),
        //             },
        //             _ => {
        //                 // do later
        //                 panic!();
        //             }
        //         }
        //     }
        //     _ => (),
        // };

        stack.pop();
        // pop stack

        loop {
            //function parsing code here
            break;
            // 'fncparse: {

            // }
            //
        }
    }

    // root.root.to_scope(HashMap::new()).execute(&functions);
}

impl Block {
    pub fn to_scope(&self, stype: ScopeType, args: HashMap<String, RefCell<Value>>) -> Scope {
        Scope {
            scopetype: ScopeType::Function,
            variables: args,
            structure: self.clone(),
            idx: 0,
        }
    }
}
// impl Scope {
//     pub fn execute(&mut self, functions: &HashMap<String, RFunction>) -> Value {
//         for fragment in &self.structure.children {}
//         Value::Null
//     }

fn evaluate_expression(expression: &Vec<ExpressionFragment>) -> Value {
    evaluate_fragment(&expression[0])
}
fn evaluate_fragment(fragment: &ExpressionFragment) -> Value {
    match fragment {
        ExpressionFragment::Constant(c) => match c {
            Constant::Bool(b) => Value::Bool(b.clone()),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Number(n) => Value::Number(n.clone()),
        },
        // ExpressionFragment::Name()
        _ => Value::Null,
    }
}
pub struct RFunction<'a> {
    pub name: String,
    pub args: Vec<String>,
    pub func: FunctionType<'a>,
}
impl RFunction<'_> {
    // pub fn call(&self, args: Vec<Value>, functions: &HashMap<String, RFunction>) -> Value {
    //     match &self.func {
    //         FunctionType::ExternalFunction(extfn) => unsafe {
    //             return extfn(args);
    //         },
    //         FunctionType::InternalFunction(intfn) => {
    //             let mut passedargs = HashMap::new();
    //             for (i, argname) in intfn.args.iter().enumerate() {
    //                 passedargs.insert(argname.clone(), Rc::new(args[i].clone()));
    //             }
    //             // return intfn.source.to_scope(passedargs).execute(&functions);
    //         }
    //     }
    // }
}
pub enum FunctionType<'a> {
    InternalFunction(parser::Function),
    ExternalFunction(libloading::Symbol<'a, unsafe extern "C" fn(Vec<Value>) -> Value>), // whattttt
}
#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, RefCell<Value>>,
    structure: Block,
    idx: usize,
    scopetype: ScopeType,
}
#[derive(Debug, Clone)]

enum ScopeType {
    If,
    Block,
    Function,
    Loop,
}
#[derive(Debug, Clone)]
#[repr(C)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
impl Value {
    pub fn to_bool(&self) -> bool {
        match &self {
            Self::Bool(b) => return *b,
            _ => panic!(),
        }
    }
    pub fn change(&mut self, val: Value) {
        *self = val;
    }
}
