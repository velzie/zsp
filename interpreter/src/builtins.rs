use std::collections::HashMap;

use std::io::{stdin, stdout, Write};
use zsp_macros::function;

use crate::runtime::{FunctionType, RFunction, Value};

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
pub fn functions<'a>() -> HashMap<String, RFunction<'a>> {
    HashMap::from([
        func!("put", zsp_put, 1),
        func!("get", zsp_get, 0),
        func!("panic", zsp_panic, 0),
        func!("assert", zsp_assert, 2),
    ])
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
