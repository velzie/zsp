use std::io::{stdin, stdout, Write};
#[no_mangle]
pub extern "C" fn std_put(inp: Vec<Value>) -> Value {
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

    if let Ok(f) = s.parse::<f64>() {
        Value::Number(f)
    } else if let Ok(b) = s.parse::<bool>() {
        Value::Bool(b)
    } else {
        Value::String(s)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
