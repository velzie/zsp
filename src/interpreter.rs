use std::{
    borrow::Borrow, cell::RefCell, collections::HashMap, f32::consts::PI, fs, ops::DerefMut, rc::Rc,
};

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

    run_root(
        root.root.to_scope(ScopeType::Block, HashMap::new()),
        &functions,
    );
    // root.root.to_scope(HashMap::new()).execute(&functions);
}
fn run_root(root: Scope, functions: &HashMap<String, RFunction>) -> Value {
    let mut stack: Vec<Scope> = vec![];
    stack.push(root);
    'stack: while stack.len() > 0 {
        let mut pointer = stack.last_mut().unwrap();
        while pointer.idx < pointer.structure.children.len() {
            // dbg!(&pointer);
            let frag = &pointer.structure.children[pointer.idx];

            match frag {
                Fragment::If {
                    condition,
                    trueblock,
                    falseblock,
                } => {
                    pointer.idx += 1;
                    if pointer.evaluate_expression(condition, functions).to_bool() {
                        let tscope = trueblock.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(tscope);
                        continue 'stack;
                    } else if let Some(fb) = falseblock {
                        let fscope = fb.to_scope(ScopeType::If, pointer.variables.clone());
                        // dbg!(&fscope);
                        stack.push(fscope);
                        continue 'stack;
                    }
                }
                Fragment::Call(call) => {
                    pointer.eval_call(call, functions);

                    // evaluate_expression(cal);
                }
                Fragment::Assignment { name, value } => {
                    if pointer.variables.contains_key(name) {
                        *pointer.variables.get_mut(name).unwrap().borrow_mut() =
                            pointer.evaluate_fragment(&value[0], functions);
                        // this feels very wrong
                        // buuuut it compiles
                    } else {
                        pointer.variables.insert(
                            name.clone(),
                            RefCell::new(pointer.evaluate_expression(value, functions)),
                        );
                    }
                }
                // Fragment::Block(block) => {
                //     let scope = block.to_scope(ScopeType::Block, pointer.variables.clone());
                //     stack.push(scope);
                //     continue 'stack;
                // }
                _ => {}
            }
            pointer.idx += 1;
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
        // dbg!("popped stack");
        stack.pop();
        // pop stack
    }
    Value::Null
}

impl Block {
    pub fn to_scope(&self, stype: ScopeType, args: HashMap<String, RefCell<Value>>) -> Scope {
        Scope {
            scopetype: stype,
            variables: args,
            structure: self.clone(),
            idx: 0,
        }
    }
}
impl Scope {
    pub fn evaluate_expression(
        &self,
        expression: &Vec<ExpressionFragment>,
        functions: &HashMap<String, RFunction>,
    ) -> Value {
        self.evaluate_fragment(&expression[0], functions)
    }
    pub fn evaluate_fragment(
        &self,
        fragment: &ExpressionFragment,
        functions: &HashMap<String, RFunction>,
    ) -> Value {
        match fragment {
            ExpressionFragment::Constant(c) => match c {
                Constant::Bool(b) => Value::Bool(b.clone()),
                Constant::String(s) => Value::String(s.clone()),
                Constant::Number(n) => Value::Number(n.clone()),
            },
            ExpressionFragment::Call(call) => self.eval_call(call, &functions),
            ExpressionFragment::Name(name) => self.variables.get(name).unwrap().borrow().clone(),
            // pass by value ^
            _ => Value::Null,
        }
    }
    pub fn eval_call(&self, call: &parser::Call, functions: &HashMap<String, RFunction>) -> Value {
        let rf = functions.get(&call.name).unwrap();
        let args: Vec<Value> = call
            .args
            .iter()
            .map(|f| self.evaluate_expression(f, &functions))
            .collect();
        self.call_function(rf, args, functions)
    }
    pub fn call_function(
        &self,
        rf: &RFunction,
        args: Vec<Value>,
        functions: &HashMap<String, RFunction>,
    ) -> Value {
        match &rf.func {
            FunctionType::ExternalFunction(extfnc) => unsafe { extfnc(args) },
            FunctionType::InternalFunction(func) => {
                let mut passedargs = self.variables.clone();
                for (i, argname) in func.args.iter().enumerate() {
                    passedargs.insert(argname.clone(), RefCell::new(args[i].clone()));
                }
                let nsc = func.source.to_scope(ScopeType::Function, passedargs);
                run_root(nsc, functions)
            }
        }
    }
}
//     pub fn execute(&mut self, functions: &HashMap<String, RFunction>) -> Value {
//         for fragment in &self.structure.children {}
//         Value::Null
//     }

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
            Self::Number(n) => {
                return n > &0.0f64;
            }
            Self::Null => return false,
            Self::String(s) => {
                return true;
                // if let Ok(b) = s.parse::<bool>() {
                //     return b;
                // } else {
                //     return false;
                // }
            }
        }
    }
}
