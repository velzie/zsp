use std::{collections::HashMap, hash::Hash, ptr::NonNull, rc::Rc};

use crate::parser::{self, Block, Constant, ExpressionFragment, Fragment, Function, Root};
use typed_arena::Arena;
pub fn interpret(root: Root) {
    // dbg!(root);
    // root.root.Run();
    // insert environemnt variables

    let mut functions = HashMap::new();

    // load dlls

    unsafe {
        let heap = Arena::new();
        // yay entire program is unsafe wooo
        for lib in root.libraries {
            // for root
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

        root.root.to_scope(HashMap::new()).execute(&functions);
    }
}

impl Block {
    pub fn to_scope(&self, args: HashMap<String, Rc<Value>>) -> Scope {
        Scope {
            variables: args,
            structure: self.clone(),
        }
    }
}
impl Scope {
    pub unsafe fn execute(&mut self, functions: &HashMap<String, RFunction>) -> Value {
        for fragment in &self.structure.children {
            match fragment {
                // Fragment::Break=>{
                //     match  {

                //     }
                // }
                Fragment::If {
                    trueblock,
                    falseblock,
                    condition,
                } => {
                    if evaluate_expression(&condition).to_bool() {
                        trueblock.to_scope(HashMap::new()).execute(&functions);
                    } else {
                        match falseblock {
                            Some(b) => b.to_scope(HashMap::new()).execute(&functions),
                            None => Value::Null,
                        };
                    }
                }
                Fragment::Assignment { name, value } => {
                    self.variables
                        .insert(name.clone(), Rc::new(evaluate_expression(value)));
                }
                Fragment::InvokeExpression(exp) => {
                    match &exp {
                        ExpressionFragment::Call { name, args } => match functions.get(name) {
                            Some(func) => {
                                func.call(
                                    args.iter().map(|f| evaluate_expression(f)).collect(),
                                    functions,
                                );
                            }
                            None => panic!(),
                        },
                        _ => {
                            // do later
                            panic!();
                        }
                    }
                }
                _ => (),
            };
        }
        Value::Null
    }
}

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
    pub unsafe fn call(&self, args: Vec<Value>, functions: &HashMap<String, RFunction>) -> Value {
        match &self.func {
            FunctionType::ExternalFunction(extfn) => {
                return extfn(args);
            }
            FunctionType::InternalFunction(intfn) => {
                let mut passedargs = HashMap::new();
                for (i, argname) in intfn.args.iter().enumerate() {
                    passedargs.insert(argname.clone(), Rc::new(args[i].clone()));
                }
                return intfn.source.to_scope(passedargs).execute(&functions);
            }
        }
    }
}
pub enum FunctionType<'a> {
    InternalFunction(parser::Function),
    ExternalFunction(libloading::Symbol<'a, unsafe extern "C" fn(Vec<Value>) -> Value>),
}
#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Rc<Value>>,
    structure: Block,
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
}
