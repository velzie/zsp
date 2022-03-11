#[macro_use]
extern crate lazy_static;
#[allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;
use substring::Substring;
mod z_std;

lazy_static! {
    static ref VARIABLES: Mutex<HashMap<String, Variable>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let contents = fs::read_to_string(&args[1]).expect("could not read file");
        interpret(&contents);
    } else {
        panic!("no file provided");
    }
}
fn interpret(file: &str) {
    let mut input = String::from(file);
    while input.len() > 0 {
        let remainder = consume(&input);
        input = remainder.clone();
    }
}
fn consume(line: &str) -> String {
    //when consume is called, we are expected to be at an empty block
    let top: Vec<&str> = line.split(" ").collect();
    let word: &str = top[0];
    let vars = VARIABLES.lock().unwrap();
    // println!("{}", word);
    match word {
        _ => {
            if z_std::std.get().contains_key(word) {
                drop(vars);
                return consume_std(line);
            } else if vars.contains_key(word) {
                drop(vars);
                return consume_variable(word, line);
            }
            println!("{} is a undefined keyword", word);
            return "".to_string();
            // panic!("{} is a undefined keyword", word);
        }
    }
}
fn consume_std(code: &str) -> String {
    match code.find("\n") {
        Some(index) => {
            let line = code.substring(0, index);
            let remainder = code.substring(index + 1, code.len());

            let lineindex = code.find(" ").unwrap();
            let keyword = line.substring(0, lineindex);
            let argstring: &str = line.substring(lineindex + 1, line.len());
            let args: Vec<&str> = argstring.split(",").collect();

            match keyword {
                "print" => println!("from code: {}", evaluate_expression(args[0])),
                _ => println!("keyword {} is not defined", keyword),
            }
            return remainder.to_string();
        }
        None => "".to_string(),
    }
}
fn evaluate_expression(expression: &str) -> Variable {
    println!("evaluating the expression {}", expression);
    let args: Vec<&str> = expression.split(" ").collect();
    let mut evaluated: String = String::default();
    let mut vars = VARIABLES.lock().unwrap();

    let mut varbuffer: Vec<Variable> = Vec::new();
    for arg in args {
        if vars.contains_key(arg) {
            varbuffer.push(vars.get(arg).unwrap().clone());
        } else if match arg.parse::<i32>() {
            Ok(n) => {
                varbuffer.push(Variable::Int(n));
                true
            }
            Err(_) => false,
        } {
        } else {
            match arg.chars().nth(0).unwrap() {
                '"' => varbuffer.push(Variable::Str(
                    arg.to_string().substring(1, arg.len() - 1).to_string(),
                )),
                _ => throw(&format!("couldn't find expression {}", arg)),
            }
        }
    }
    return Variable::Str(evaluated);
}
fn throw(err: &str) {
    println!("error occured when parsing {}", err)
}
#[derive(Debug, Clone)]
enum Variable {
    Str(String),
    Int(i32),
    Bool(bool),
    None,
}

impl std::fmt::Display for Variable {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match *self {
            Variable::Str(ref string) => write!(f, "{}", string),
            Variable::Int(ref int) => write!(f, "{}", int),
            Variable::Bool(ref bool) => write!(f, "{}", bool),
            Variable::None => write!(f, "{}", "None"),
        }
    }
}
