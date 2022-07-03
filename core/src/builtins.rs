use std::collections::HashMap;

use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use substring::Substring;
use zsp_macros::function;

use crate::runtime::{FunctionType, Object, RFunction, Value};

macro_rules! func {
    ($name:expr,$fn:expr,$args:expr) => {
        (
            String::from($name),
            RFunction {
                name: String::from($name),
                func: FunctionType::BuiltinFunction($fn),
                args: vec!["".into(); $args],
            },
        )
    };
}
macro_rules! afunc {
    ($name:expr,$fn:expr,$args:expr) => {
        (
            String::from($name),
            Rc::new(RefCell::new(Value::Lambda(RFunction {
                name: String::from($name),
                func: FunctionType::BuiltinFunction($fn),
                args: vec!["".into(); $args],
            }))),
        )
    };
}
pub fn functions<'a>() -> HashMap<String, RFunction<'a>> {
    HashMap::from([
        func!("put", zsp_put, 1),
        func!("get", zsp_get, 0),
        func!("exit", zsp_exit, 0),
        func!("assert", zsp_assert, 2),
        func!("array", zsp_array_new, 0),
        func!("object", zsp_object_new, 0),
        func!("len", zsp_array_len, 1),
        func!("null", zsp_null_new, 0),
        func!("panic", zsp_panic, 0),
    ])
}

pub fn boolprototype<'a>() -> HashMap<String, Rc<RefCell<Value<'a>>>> {
    HashMap::new()
}
pub fn stringprototype<'a>() -> HashMap<String, Rc<RefCell<Value<'a>>>> {
    HashMap::from([afunc!("firstchar", string_substr, 0)])
}
pub fn arrayprototype<'a>() -> HashMap<String, Rc<RefCell<Value<'a>>>> {
    HashMap::new()
}
pub fn numberprototype<'a>() -> HashMap<String, Rc<RefCell<Value<'a>>>> {
    HashMap::from([afunc!("add", number_add, 1)])
}
fn number_add(inp: Vec<Value>) -> Value {
    Value::Number(inp[0].to_number() + inp[1].to_number())
}
fn string_substr(inp: Vec<Value>) -> Value {
    Value::String(inp[0].to_string().substring(0, 1).to_string())
}

fn zsp_array_new(_args: Vec<Value>) -> Value {
    Value::Array(vec![])
}

pub fn zsp_object_new(_args: Vec<Value>) -> Value {
    Value::Object(Object {
        fields: HashMap::new(),
    })
}
pub fn zsp_null_new(_args: Vec<Value>) -> Value {
    Value::Null
}
fn zsp_array_len(args: Vec<Value>) -> Value {
    Value::Number(args[0].clone().as_array().len() as f32)
}

fn zsp_put(args: Vec<Value>) -> Value {
    println!("{}", args[0].to_string());
    Value::Null
}

fn zsp_get(_inp: Vec<Value>) -> Value {
    stdout().flush().unwrap();
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("you didn't enter a string");
    s = s.chars().filter(|c| c != &'\n' && c != &'\r').collect();
    if let Ok(f) = s.parse::<f32>() {
        Value::Number(f)
    } else if let Ok(b) = s.parse::<bool>() {
        Value::Bool(b)
    } else {
        Value::String(s)
    }
}
fn zsp_exit(_args: Vec<Value>) -> Value {
    dbg!("exiting");
    std::process::exit(0);
}
fn zsp_panic(_args: Vec<Value>) -> Value {
    panic!("panic from code")
}
fn zsp_assert(inp: Vec<Value>) -> Value {
    if inp[0] != inp[1] {
        panic!("FAILED ASSERT: {:?} not equal to {:?}", inp[0], inp[1]);
    }
    Value::Null
}
