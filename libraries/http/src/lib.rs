use std::collections::HashMap;
use std::io::Read;
use zsp_core::exceptions::Exception;
use zsp_core::func;
use zsp_core::runtime::{FunctionType, RFunction, Value};
#[no_mangle]
pub fn lib() -> HashMap<String, RFunction> {
    HashMap::from([func!("http_get", get, 1)])
}

#[no_mangle]
pub fn get<'a>(inp: Vec<Value<'a>>) -> Result<Value<'a>, Exception> {
    match reqwest::blocking::get(inp[0].to_string()) {
        Ok(mut e) => {
            let mut buf = String::new();
            match e.read_to_string(&mut buf) {
                Ok(_) => Ok(Value::String(buf)),
                Err(e) => Err(Exception::new(
                    2,
                    "HttpGetError",
                    &format!("Error in http request: {}", e),
                )),
            }
        }
        Err(e) => Err(Exception::new(
            2,
            "HttpGetError",
            &format!("Error in http request: {}", e),
        )),
    }
}
