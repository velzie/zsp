extern crate zsp_interpreter;
use std::io::{stdin, stdout, Write};

use std::ffi;

use zsp_interpreter::runtime::Value;
// use ffi_std::
// use std::hash::*
#[no_mangle]
pub extern "C" fn std_put(inp: Vec<Value>) -> Value {
    // zsp_interpreter::;
    println!(
        "{}",
        match &inp[0] {
            Value::String(s) => s.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::from("null"),
        }
    );

    Value::Null
}
#[no_mangle]
pub extern "C" fn std_get(inp: Vec<Value>) -> Value {
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
#[no_mangle]
pub extern "C" fn std_panic() {
    panic!("panic from code");
}
#[no_mangle]

pub extern "C" fn std_assert(inp: Vec<Value>) {
    if inp[0] != inp[1] {
        panic!("FAILED ASSERT: {:?} not equal to {:?}", inp[0], inp[1]);
    }
}
