use std::{cell::RefCell, collections::HashMap, fmt::Debug, fs, path::Path, rc::Rc};

use home::home_dir;

use crate::{
    builtins,
    exceptions::Exception,
    lexer::{self, Op},
    parser::{self, Block, Constant, ExpressionFragment, Frag, VarRef, VarRefFragment, VarRefRoot},
};

// inp structure ideas:
//1: simplest possible solutions
// single function for parsing a scope, doesn't return but instead mutates a scope, when a return is called

// or

// returns an enum, none for no return and if there is a return keep going back on the stack until a function block is hit

//2: undefined behavior:
// self referential struct with parent owning child, store return value within struct and iterate from there

//3. pointers

// run a loop with a linked list of pointer to the block, shift back one pointer when the block is exited, retain
pub fn execute<'a>(
    input: &String,
    envfncs: Option<HashMap<String, RFunction>>,
) -> Result<(), Exception> {
    let mut tokens = lexer::lex(input.clone());
    // println!("{:?}", tokens);
    let libnames = parser::find_loads(&mut tokens)?;

    let mut libraryfunctions = HashMap::new();
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    for ln in libnames {
        unsafe {
            let pathv = format!("{}/.zsp", home_dir().unwrap().display());
            let zspdir = Path::new(&pathv);
            fs::create_dir_all(zspdir).unwrap();
            let lib = Box::leak(Box::new(
                libloading::Library::new(format!(
                    "{}/{}",
                    pathv,
                    ln //+ .dll or .so
                ))
                .unwrap(),
            ));
            // produces a 'static reference. needed as library will survive for the entirety of the program
            let libinfo = lib
                .get::<fn() -> HashMap<String, RFunction>>("lib".as_bytes())
                .unwrap()();

            for (k, v) in libinfo.into_iter() {
                libraryfunctions.insert(k, v);
            }
        }
    }
    let root = parser::parse(tokens, &input, &libraryfunctions)?;
    // println!("{:?}", root);

    let mut functions = builtins::functions();

    if let Some(ext) = envfncs.clone() {
        for (k, v) in ext.into_iter() {
            libraryfunctions.insert(k, v);
        }
    }
    for (k, v) in libraryfunctions {
        functions.insert(k.clone(), v.clone());
    }
    for fun in root.functions {
        let cfn = fun.1.clone();
        functions.insert(
            fun.0,
            RFunction {
                args: cfn.args.clone(),
                func: FunctionType::InternalFunction(cfn.clone()),
            },
        );
    }

    let rootscope = Rc::new(RefCell::new(
        root.root.to_scope(ScopeType::Function, HashMap::new()),
    ));
    run_root(rootscope, &functions, &input)?;

    // rootscope.call_function(&functions.get("draw").unwrap(), vec![], &functions, &input);
    Ok(())
}

pub fn run_root<'a>(
    root: Rc<RefCell<Scope<'a>>>,
    functions: &HashMap<String, RFunction>,
    input: &String,
) -> Result<Value<'a>, Exception> {
    let mut stack: Vec<Rc<RefCell<Scope>>> = vec![];
    stack.push(root);
    'stack: while stack.len() > 0 {
        let pointerc = stack.last_mut().unwrap().clone();
        let mut pointer = pointerc.borrow_mut();
        match &pointer.scopetype {
            ScopeType::For {
                condition,
                incrementor: _,
            } => {
                if !pointer
                    .evaluate_expression(condition, &functions, input, 0)?
                    .to_bool()
                {
                    stack.pop();
                    continue 'stack;
                }
            }
            _ => (),
        }
        while pointer.idx < pointer.structure.children.len() {
            // dbg!(&pointer);
            let frag = &pointer.structure.children[pointer.idx].clone();

            match &frag.frag {
                Frag::For {
                    name,
                    initial,
                    condition,
                    incrementor,
                    block,
                } => {
                    let mut scope = block.to_scope(
                        ScopeType::For {
                            condition: condition.clone(),
                            incrementor: incrementor.clone(),
                        },
                        pointer.variables.clone(),
                    );
                    let v = Rc::new(RefCell::new(
                        pointer.evaluate_expression(initial, &functions, input, frag.index)?,
                    ));
                    scope.variables.insert(name.clone(), v);
                    pointer.idx += 1;
                    stack.push(Rc::new(RefCell::new(scope)));
                    continue 'stack;
                }
                Frag::If {
                    condition,
                    trueblock,
                    falseblock,
                } => {
                    if pointer
                        .evaluate_expression(&condition, functions, input, frag.index)?
                        .to_bool()
                    {
                        pointer.idx += 1;
                        let tscope = trueblock.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(Rc::new(RefCell::new(tscope)));
                        continue 'stack;
                    } else if let Some(fb) = falseblock {
                        pointer.idx += 1;
                        let fscope = fb.to_scope(ScopeType::If, pointer.variables.clone());
                        stack.push(Rc::new(RefCell::new(fscope)));
                        continue 'stack;
                    }
                }
                Frag::Call(call) => {
                    // note: this will evaluate the call. i would prefer it to be a little more explicit but that would just mean repeating code i already wrote
                    pointer.get_varref(call, false, functions, input, frag.index)?;
                }
                Frag::Assignment { variable, value } => {
                    // dbg!(pointer
                    //     .get_varref(variable.clone(), true, functions, input, frag.index)
                    //     .borrow_mut());
                    *pointer
                        .get_varref(variable, true, functions, input, frag.index)?
                        .borrow_mut() =
                        pointer.evaluate_expression(&value, functions, input, frag.index)?;
                }
                Frag::Initialize { variable, value } => {
                    let v = pointer.evaluate_expression(&value, functions, input, frag.index);
                    pointer
                        .variables
                        .insert(variable.clone(), Rc::new(RefCell::new(v?)));
                }
                Frag::Return(exp) => {
                    return Ok(pointer.evaluate_expression(&exp, functions, input, frag.index)?);
                }
                Frag::Break => {
                    let idx = frag.index.clone();
                    match stack.clone().iter().rev().position(|f| {
                        stack.pop();
                        match f.borrow_mut().scopetype {
                            ScopeType::Loop => return true,
                            _ => return false,
                        }
                    }) {
                        Some(_) => {
                            // idx += 1;
                        }
                        None => {
                            return Err(Exception::new(
                                idx,
                                "InvalidBreakException",
                                "you can only break in a loop. this doesn't look like a loop",
                            ))
                        }
                    };
                    // frag.clone();
                    continue 'stack;
                }
                Frag::Loop(block) => {
                    let scope = block.to_scope(ScopeType::Loop, pointer.variables.clone());

                    pointer.idx += 1;
                    stack.push(Rc::new(RefCell::new(scope)));
                    continue 'stack;
                }
                Frag::Block(block) => {
                    let scope = block.to_scope(ScopeType::Block, pointer.variables.clone());
                    pointer.idx += 1;
                    stack.push(Rc::new(RefCell::new(scope)));
                    continue 'stack;
                }
            }
            pointer.idx += 1;
        }
        match &pointer.scopetype {
            ScopeType::Loop => {
                pointer.idx = 0;
            }
            ScopeType::For {
                condition: _,
                incrementor,
            } => {
                run_root(
                    Rc::new(RefCell::new(
                        incrementor.to_scope(ScopeType::Block, pointer.variables.clone()),
                    )),
                    functions,
                    input,
                )?;
                pointer.idx = 0;
            }
            _ => {
                stack.pop();
            }
        }
        // pop stack
    }
    Ok(Value::Null)
}

impl<'a> Block {
    pub fn to_scope(
        &self,
        stype: ScopeType,
        args: HashMap<String, Rc<RefCell<Value<'a>>>>,
    ) -> Scope<'a> {
        Scope {
            scopetype: stype,
            variables: args,
            structure: self.clone(),
            idx: 0,
        }
    }
}
impl<'a> Scope<'a> {
    pub fn evaluate_expression(
        &self,
        expression: &Vec<ExpressionFragment>,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<Value<'a>, Exception> {
        let mut vals = vec![];
        for f in expression {
            vals.push(match f {
                ExpressionFragment::Op(op) => ExpressionVal::Op(op.clone()),
                _ => ExpressionVal::Value(self.evaluate_fragment(f, functions, input, indexptr)?),
            })
        }
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

                        Op::Or => Value::Bool(buffer.to_bool() || v.to_bool()),
                        Op::And => Value::Bool(buffer.to_bool() && v.to_bool()),

                        Op::EqualTo => Value::Bool(buffer == v),
                        Op::NotEqualTo => Value::Bool(buffer.to_number() != v.to_number()),
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
                        Op::Power => Value::Number(buffer.to_number().powf(v.to_number())),
                    }
                }
            }
        }

        // todo: implement order of operations, or at least ()

        Ok(buffer)
        // self.evaluate_fragment(&expression[0], functions)
    }
    pub fn evaluate_fragment(
        &self,
        frag: &ExpressionFragment,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<Value<'a>, Exception> {
        match frag {
            ExpressionFragment::Constant(c) => match c {
                Constant::Bool(b) => Ok(Value::Bool(b.clone())),
                Constant::String(s) => Ok(Value::String(s.clone())),
                Constant::Number(n) => Ok(Value::Number(n.clone())),
            },
            ExpressionFragment::Call(call) => self.eval_call(call, &functions, input, indexptr),
            // ExpressionFragment::Name(name) => {
            //     self.variables.get(name).unwrap().borrow_mut().clone()
            // }
            ExpressionFragment::Expression(expr) => {
                Ok(self.evaluate_expression(expr, functions, input, indexptr)?)
            }
            ExpressionFragment::VarRef(vref) => {
                let vref = self.get_varref(vref, false, functions, input, indexptr)?;
                let tmp = vref.clone();
                let deref = tmp.borrow_mut();
                Ok(match *deref {
                    Value::Object(_) | Value::DynObject(_) => Value::Reference(vref), // passes by references
                    _ => deref.clone(), // else passes by value
                })
            }
            ExpressionFragment::Lambda(l) => Ok(Value::Lambda {
                takeself: false,
                func: RFunction {
                    args: l.args.clone(),
                    func: FunctionType::InternalFunction(l.clone()),
                },
            }),
            ExpressionFragment::Op(_)=>{
                return Err(Exception::new(
                    indexptr,
                    "UnreachableUnexpectedSymbolException",
                    "Cannot use an operator (+,-,/,*,etc...) in an expression fragment. this shouldn't be reachable unless either you or i messed up really badly",
                ))
            }
        }
    }
    pub fn call_lambda(
        &self,
        func: &Value,
        expargs: &Vec<Vec<ExpressionFragment>>,
        ptr: &mut Rc<RefCell<Value<'a>>>,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<(), Exception> {
        match func {
            Value::Lambda { takeself, func } => {
                // TODO: make it not clone, use &mut references instead
                let mut args = vec![];
                for arg in expargs {
                    args.push(self.evaluate_expression(arg, &functions, input, indexptr)?);
                }
                let mut nargs = vec![];
                let mut requiredargs = func.args.len();
                if *takeself {
                    requiredargs += 1;
                    nargs.push(ptr.borrow_mut().clone());
                }
                nargs.append(&mut args);

                if nargs.len() != requiredargs {
                    return Err(Exception::new(
                        indexptr,
                        "ArgumentException",
                        &format!("expected {} args, got {} args", requiredargs, nargs.len()),
                    ));
                }

                *ptr = Rc::new(RefCell::new(
                    self.call_function(func, nargs, functions, input)?,
                ));
                Ok(())
            }
            _ => {
                return Err(Exception::new(
                    indexptr,
                    "IncorrectTypeException",
                    "cannot call something that is not a function",
                ));
            }
        }
    }
    pub fn get_varref(
        &self,
        varref: &VarRef,
        assign: bool,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<Rc<RefCell<Value<'a>>>, Exception> {
        let mut ptr = self.get_varref_root(&varref.root, functions, input, indexptr)?;
        for i in 0..varref.operations.len() {
            let op = &varref.operations[i];
            match op {
                VarRefFragment::IndexInto(ind) => {
                    let index = self
                        .evaluate_expression(ind, functions, input, indexptr)?
                        .to_number() as usize;

                    let ar = ptr.clone().borrow_mut().clone();
                    match ar {
                        Value::Array(_) => {
                            if assign && i == varref.operations.len() - 1 {
                                let mut mutval = ptr.borrow_mut();
                                let arr = mutval.as_array();
                                while index >= arr.len() {
                                    arr.push(Rc::new(RefCell::new(Value::Null)));
                                }
                            }
                            let tmp = ptr.borrow_mut().as_array()[index].clone();
                            ptr = tmp;
                        }
                        _ => {
                            return Err(Exception::new(
                                indexptr,
                                "InvalidIndexException",
                                "can only index into an array",
                            ))
                        }
                    }
                }
                VarRefFragment::ObjectCall { name, args } => {
                    let fields = ptr.borrow_mut().fields();

                    match fields.get(name) {
                        Some(v) => {
                            let val = v.borrow_mut();
                            self.call_lambda(&val, args, &mut ptr, functions, input, indexptr)?;
                        }
                        None => {
                            return Err(Exception::new(
                                indexptr,
                                "FieldNotFoundException",
                                &format!("Object does not have a field named {}", name),
                            ))
                        }
                    }
                }
                VarRefFragment::ObjectProperty(name) => {
                    let mut tmp = ptr.borrow_mut().clone();
                    match &tmp {
                        Value::Reference(_) => ptr = tmp.as_ref().clone(), // automatic derefing.
                        _ => (),
                    }
                    let fields = ptr.borrow_mut().fields();
                    match fields.get(name) {
                        Some(v) => {
                            ptr = v.clone();
                        }
                        None => {
                            let tmp = ptr.clone();
                            let mut ptrref = tmp.borrow_mut();
                            if assign && ptrref.is_object() {
                                let val = Rc::new(RefCell::new(Value::Null));
                                ptrref.as_object().fields.insert(name.clone(), val.clone());
                                ptr = val;
                            };
                        }
                    }
                }
                VarRefFragment::LambdaCall(l) => {
                    let tmp = ptr.clone();
                    let mut fnc = tmp.borrow_mut();
                    self.call_lambda(&mut fnc, l, &mut ptr, functions, input, indexptr)?
                }
            }
        }
        Ok(ptr)
    }
    pub fn get_varref_root(
        &self,
        root: &VarRefRoot,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<Rc<RefCell<Value<'a>>>, Exception> {
        match root {
            VarRefRoot::Call(c) => match self.eval_call(&c, functions, input, indexptr) {
                Ok(u) => Ok(Rc::new(RefCell::new(u))),
                Err(e) => return Err(e),
            },
            VarRefRoot::Variable(v) => match self.variables.get(v) {
                Some(s) => Ok(s.clone()),
                None => {
                    return Err(Exception::new(
                        indexptr,
                        "VariableNotFoundException",
                        &format!(
                            "variable with name {} was not found in the current scope",
                            &v
                        ),
                    ))
                }
            },
        }
    }
    pub fn eval_call(
        &self,
        call: &parser::Call,
        functions: &HashMap<String, RFunction>,
        input: &String,
        indexptr: usize,
    ) -> Result<Value<'a>, Exception> {
        let rf = functions.get(&call.name).unwrap();
        let mut args = vec![];
        for arg in &call.args {
            args.push(self.evaluate_expression(&arg, &functions, input, indexptr)?);
        }
        self.call_function(rf, args, functions, input)
    }
    pub fn call_function(
        &self,
        rf: &RFunction,
        args: Vec<Value<'a>>,
        functions: &HashMap<String, RFunction>,
        input: &String,
    ) -> Result<Value<'a>, Exception> {
        match &rf.func {
            FunctionType::InternalFunction(func) => {
                let mut passedargs = self.variables.clone();
                for (i, argname) in func.args.iter().enumerate() {
                    passedargs.insert(argname.clone(), Rc::new(RefCell::new(args[i].clone())));
                }
                let nsc = func.source.to_scope(ScopeType::Function, passedargs);
                run_root(Rc::new(RefCell::new(nsc)), functions, input)
            }
            FunctionType::ExternalFunction(func) => func(args),
        }
    }
}
enum ExpressionVal<'a> {
    Value(Value<'a>),
    Op(Op),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RFunction {
    pub args: Vec<String>,
    pub func: FunctionType,
}

#[derive(Clone)]
pub enum FunctionType {
    InternalFunction(parser::Function),
    ExternalFunction(Extfn),
}
impl Debug for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Function>")
    }
}

impl PartialEq for FunctionType {
    fn eq(&self, _: &Self) -> bool {
        panic!("you cant compare functions")
    }
}
#[derive(Debug, Clone)]
pub struct Scope<'a> {
    variables: HashMap<String, Rc<RefCell<Value<'a>>>>,
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
    For {
        condition: Vec<ExpressionFragment>,
        incrementor: Block,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    String(String),
    Number(f32),
    Bool(bool),
    Array(Vec<Rc<RefCell<Value<'a>>>>),
    Null,
    Object(Object<'a>),
    Lambda { takeself: bool, func: RFunction },
    DynObject(DynObjectContainer<'a>),
    Reference(Rc<RefCell<Value<'a>>>),
}
pub fn downcast_dyn<'a, T>(u: &mut DynObjectContainer) -> &'a mut T {
    unsafe { &mut *(&mut *u.val as *mut dyn DynObject as *mut T) }
    // don't ask. please don't ask
}
#[derive(Debug, Clone, PartialEq)]
pub struct Object<'a> {
    pub fields: HashMap<String, Rc<RefCell<Value<'a>>>>,
}
#[derive(Debug, Clone)]
pub struct DynObjectContainer<'a> {
    pub val: Box<dyn DynObject<'a>>,
    // pub pro
}
impl<'a> PartialEq for DynObjectContainer<'a> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl<'a> Debug for Box<dyn DynObject<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug(f)
    }
}
pub trait DynObject<'a>: dyn_clone::DynClone {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn fields(&self) -> HashMap<String, Rc<RefCell<Value<'a>>>>;
}
dyn_clone::clone_trait_object!(DynObject<'_>);
impl<'a> Value<'a> {
    pub fn fields(&self) -> HashMap<String, Rc<RefCell<Value<'a>>>> {
        match self {
            Value::Bool(_) => builtins::boolprototype(),
            Value::Number(_) => builtins::numberprototype(),
            Value::Null => HashMap::new(),
            Value::String(_) => builtins::stringprototype(),
            Value::Array(_) => builtins::arrayprototype(),
            Value::Object(obj) => obj.fields.clone(),
            Value::Lambda {
                takeself: _,
                func: _,
            } => HashMap::new(),
            Value::DynObject(o) => o.val.fields(),
            Value::Reference(r) => r.borrow_mut().fields(),
        }
    }
    pub fn cast(&self, vtype: Value<'a>) -> Value<'a> {
        match vtype {
            Value::Bool(_) => match self {
                Self::Bool(_) => self.clone(),
                Self::Number(n) => Self::Bool(n > &0f32),
                Self::Null => Self::Bool(false),
                Self::String(s) => Self::Bool(s.eq("true")),
                Self::Array(_) => panic!("cannot cast array to bool"),
                _ => panic!(),
            },
            Value::String(_) => match self {
                Self::String(_) => self.clone(),
                Self::Bool(b) => Self::String(format!("{}", b)),
                Self::Number(n) => Self::String(format!("{}", n)),
                Self::Null => Self::String(String::default()),
                Self::Array(_) => panic!("cannot cast array to string"),
                _ => Self::String(format!("{:?}", self)),
            },
            Value::Number(_) => match self {
                Self::Number(_) => self.clone(),
                Self::String(s) => Self::Number(s.parse::<f32>().unwrap_or_default()),
                Self::Bool(b) => Self::Number(if *b { 1f32 } else { 0f32 }),
                Self::Null => Self::Number(0f32),
                Self::Array(_) => panic!("cannot cast array to number"),
                _ => panic!(),
            },
            Value::Null => panic!("cannot cast null"),
            Self::Array(_) => panic!("cannot cast array"),
            _ => panic!(),
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_) => true,
            _ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }
    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }
    pub fn as_ref(&mut self) -> &mut Rc<RefCell<Value<'a>>> {
        match self {
            Value::Reference(r) => r,
            _ => unreachable!(),
        }
    }
    pub fn as_object(&mut self) -> &mut Object<'a> {
        match self {
            Value::Object(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_dyn_object(&mut self) -> &mut DynObjectContainer<'a> {
        match self {
            Value::DynObject(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_array(&mut self) -> &mut Vec<Rc<RefCell<Value<'a>>>> {
        match self {
            Value::Array(a) => a,
            _ => unreachable!(),
        }
    }
    pub fn to_bool(&self) -> bool {
        match self.cast(Value::Bool(false)) {
            Value::Bool(b) => b,
            _ => unreachable!(),
        }
    }
    pub fn to_number(&self) -> f32 {
        match self.cast(Value::Number(0f32)) {
            Value::Number(b) => b,
            _ => unreachable!(),
        }
    }
    pub fn to_string(&self) -> String {
        match self.cast(Value::String(String::from("_"))) {
            Value::String(b) => b,
            _ => unreachable!(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Library<'a> {
    pub func: fn(Vec<Value<'a>>) -> Result<Value<'a>, Exception>,
}

type Extfn = fn(Vec<Value>) -> Result<Value, Exception>;
