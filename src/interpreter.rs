use std::{borrow::Borrow, cell::RefCell, collections::HashMap};

use crate::{
    exceptions::rtexception,
    lexer::Op,
    parser::{self, Block, Constant, ExpressionFragment, Frag, Function, Root},
};
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
            let libobj = heap.alloc(
                libloading::Library::new(format!(
                    "{}/{}",
                    std::env::current_dir()
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                    lib.static_loc
                ))
                .unwrap(),
            ); // lazy bad unsafe garbage code

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
        root.root.to_scope(ScopeType::Function, HashMap::new()),
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

            match &frag.frag {
                Frag::If {
                    condition,
                    trueblock,
                    falseblock,
                } => {
                    pointer.idx += 1;
                    if pointer.evaluate_expression(&condition, functions).to_bool() {
                        let tscope = trueblock.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(tscope);
                        continue 'stack;
                    } else if let Some(fb) = falseblock {
                        let fscope = fb.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(fscope);
                        continue 'stack;
                    }
                }
                Frag::Call(call) => {
                    pointer.eval_call(&call, functions);
                }
                Frag::Assignment { name, value } => {
                    if pointer.variables.contains_key(name) {
                        *pointer.variables.get_mut(name).unwrap().borrow_mut() =
                            pointer.evaluate_expression(&value, functions);
                        // this feels very wrong
                        // buuuut it compiles
                    } else {
                        pointer.variables.insert(
                            name.clone(),
                            RefCell::new(pointer.evaluate_expression(&value, functions)),
                        );
                    }
                }
                Frag::Return(exp) => {
                    return pointer.evaluate_expression(&exp, functions);
                }
                Frag::Break => {
                    let idx = frag.index.clone();
                    match stack.clone().iter().rev().position(|f| {
                        stack.pop();
                        match f.scopetype {
                            ScopeType::Loop => return true,
                            _ => return false,
                        }
                    }) {
                        Some(p) => {
                            // idx += 1;
                        }
                        None => rtexception(
                            &String::from("input"),
                            idx,
                            "InvalidBreakException",
                            "you can only break in a loop. this doesn't look like a loop",
                        ),
                    };
                    // frag.clone();
                    continue 'stack;
                }
                Frag::Loop(block) => {
                    let scope = block.to_scope(ScopeType::Loop, pointer.variables.clone());

                    pointer.idx += 1;
                    stack.push(scope);
                    continue 'stack;
                }
                Frag::Block(block) => {
                    let scope = block.to_scope(ScopeType::Block, pointer.variables.clone());
                    pointer.idx += 1;
                    stack.push(scope);
                    continue 'stack;
                }
            }
            pointer.idx += 1;
        }
        match pointer.scopetype {
            ScopeType::Loop => {
                pointer.idx = 0;
            }
            _ => {
                stack.pop();
            }
        }
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
        let mut vals: Vec<ExpressionVal> = expression
            .iter()
            .map(|f| match f {
                ExpressionFragment::Op(op) => ExpressionVal::Op(op.clone()),
                _ => ExpressionVal::Value(self.evaluate_fragment(f, functions)),
            })
            .collect();
        let mut buffer: Value = match &vals[0] {
            ExpressionVal::Value(v) => v.clone(),
            _ => panic!(),
        };
        vals.remove(0);

        let mut opr = Op::Not; //haha get it not because this never gets used because this code structure is terrible
        for val in vals {
            match val {
                ExpressionVal::Op(op) => opr = op,
                ExpressionVal::Value(v) => {
                    buffer = match opr {
                        Op::Not => {
                            dbg!("this is not the correct impl, don't rlly care");
                            Value::Bool(!v.to_bool())
                        }
                        Op::EqualTo => Value::Bool(buffer == v),
                        Op::NotEqualTo => Value::Bool(buffer.to_bool() != v.to_bool()),
                        Op::GreaterThan => Value::Bool(buffer.to_number() > v.to_number()),
                        Op::GreaterThanOrEqualTo => {
                            Value::Bool(buffer.to_number() >= v.to_number())
                        }
                        Op::LessThan => Value::Bool(buffer.to_number() < v.to_number()),
                        Op::LessThanOrEqualTo => Value::Bool(buffer.to_number() <= v.to_number()),
                        Op::Plus => Value::Number(buffer.to_number() + v.to_number()),
                        Op::Minus => Value::Number(buffer.to_number() - v.to_number()),
                        Op::Multiply => Value::Number(buffer.to_number() * v.to_number()),
                        Op::Divide => Value::Number(buffer.to_number() / v.to_number()),
                        Op::Power => Value::Number(buffer.to_number().pow(v.to_number() as u32)),
                    }
                }
            }
        }

        // todo: implement order of operations, or at least ()

        buffer
        // self.evaluate_fragment(&expression[0], functions)
    }
    pub fn evaluate_fragment(
        &self,
        frag: &ExpressionFragment,
        functions: &HashMap<String, RFunction>,
    ) -> Value {
        match frag {
            ExpressionFragment::Constant(c) => match c {
                Constant::Bool(b) => Value::Bool(b.clone()),
                Constant::String(s) => Value::String(s.clone()),
                Constant::Number(n) => Value::Number(n.clone()),
            },
            ExpressionFragment::Call(call) => self.eval_call(call, &functions),
            ExpressionFragment::Name(name) => self.variables.get(name).unwrap().borrow().clone(),
            _ => panic!(),
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
//         for Frag in &self.structure.children {}
//         Value::Null
//     }
enum ExpressionVal {
    Value(Value),
    Op(Op),
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

pub enum ScopeType {
    If,
    Block,
    Function,
    Loop,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    Null,
}
impl Value {
    pub fn cast(&self, vtype: Value) -> Value {
        match vtype {
            Value::Bool(_) => match self {
                Self::Bool(b) => self.clone(),
                Self::Number(n) => Self::Bool(n > &0),
                Self::Null => Self::Bool(false),
                Self::String(s) => Self::Bool(s.eq("true")),
            },
            Value::String(_) => match self {
                Self::String(s) => self.clone(),
                Self::Bool(b) => Self::String(format!("{}", b)),
                Self::Number(n) => Self::String(format!("{}", n)),
                Self::Null => Self::String(String::default()),
            },
            Value::Number(_) => match self {
                Self::Number(n) => self.clone(),
                Self::String(s) => Self::Number(s.parse::<i64>().unwrap_or_default()),
                Self::Bool(b) => Self::Number(if *b { 1 } else { 0 }),
                Self::Null => Self::Number(0),
            },
            Value::Null => panic!("what"),
        }
    }
    pub fn to_bool(&self) -> bool {
        match self.cast(Value::Bool(false)) {
            Value::Bool(b) => b,
            _ => panic!(),
        }
    }
    pub fn to_number(&self) -> i64 {
        match self.cast(Value::Number(0)) {
            Value::Number(b) => b,
            _ => panic!(),
        }
    }
    pub fn to_string(&self) -> String {
        match self.cast(Value::String(String::from("_"))) {
            Value::String(b) => b,
            _ => panic!(),
        }
    }
}
