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
        func!("panic", zsp_panic, 0),
        func!("assert", zsp_assert, 2),
        func!("array", zsp_array_new, 0),
        func!("object", zsp_object_new, 0),
        func!("len", zsp_array_len, 1),
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

// impl Value{
//     pub fn functions<'a>() -> HashMap<String,RFunction<'a>>{
//         HashMap
//     }
// }

fn zsp_array_new(args: Vec<Value>) -> Value {
    Value::Array(vec![])
}

pub fn zsp_object_new(args: Vec<Value>) -> Value {
    Value::Object(Object {
        fields: HashMap::new(),
    })
}
fn zsp_array_len(args: Vec<Value>) -> Value {
    Value::Number(args[0].clone().as_array().len() as i64)
}

fn zsp_put(args: Vec<Value>) -> Value {
    println!("{}", args[0].to_string());
    Value::Null
}

fn zsp_get(inp: Vec<Value>) -> Value {
    stdout().flush();
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("you didn't enter a string");

    if let Ok(f) = s.parse::<i64>() {
        Value::Number(f)
    } else if let Ok(b) = s.parse::<bool>() {
        Value::Bool(b)
    } else {
        Value::String(s)
    }
}
fn zsp_panic(args: Vec<Value>) -> Value {
    panic!("panic from code");
}
fn zsp_assert(inp: Vec<Value>) -> Value {
    if inp[0] != inp[1] {
        panic!("FAILED ASSERT: {:?} not equal to {:?}", inp[0], inp[1]);
    }
    Value::Null
}
