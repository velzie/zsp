#[macro_use]
extern crate lazy_static;
#[allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;
use substring::Substring;
// mod z_std;
// use z_std::Std;

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
            if true {
                drop(vars);
                return consume_std(line);
            } else if vars.contains_key(word) {
                drop(vars);
            }
            println!("{} is a undefined keyword", word);
            return "".to_string();
            // panic!("{} is a undefined keyword", word);
        }
    }
}
fn consume_std(code: &str) -> String {
    let code: (String, String) = split_string(code, '\n');

    let kargs = split_string(&code.0, ' ');
    let args: Vec<&str> = kargs.1.split(",").collect();

    match kargs.0.as_str() {
        "print" => println!("from code: {}", evaluate_expression(args[0])),
        _ => println!("keyword {} is not defined", kargs.0),
    }
    return code.1;
}
fn split_string(input: &str, character: char) -> (String, String) {
    match input.find(character) {
        Some(index) => (
            input.substring(0, index).to_string(),
            input.substring(index + 1, input.len()).to_string(),
        ),
        None => (input.to_string(), String::default()),
    }
}

fn evaluate_expression(expression: &str) -> Variable {
    println!("evaluating the expression {}", expression);
    let args: Vec<&str> = expression.split(" ").collect();
    let mut evaluated: Variable;
    let vars = VARIABLES.lock().unwrap();

    let mut varbuffer: Vec<PreExpression> = Vec::new();
    for arg in args {
        if vars.contains_key(arg) {
            varbuffer.push(PreExpression::Variable(vars.get(arg).unwrap().clone()));
        } else if match arg.parse::<i32>() {
            Ok(n) => {
                varbuffer.push(PreExpression::Variable(Variable::Int(n)));
                true
            }
            Err(_) => false,
        } {
        } else if match arg {
            "+" => {
                varbuffer.push(PreExpression::PLUS);
                true
            }
            "-" => {
                varbuffer.push(PreExpression::MINUS);
                true
            }
            "*" => {
                varbuffer.push(PreExpression::MULTIPLY);
                true
            }
            "/" => {
                varbuffer.push(PreExpression::DIVIDE);
                true
            }
            _ => false,
        } {
        } else {
            match arg.chars().nth(0).unwrap() {
                '"' => varbuffer.push(PreExpression::Variable(Variable::Str(
                    arg.to_string().substring(1, arg.len() - 1).to_string(),
                ))),
                _ => throw(&format!("couldn't find expression {}", arg)),
            }
        }
    }
    evaluated = match varbuffer[0] {
        PreExpression::Variable(ref v) => v.clone(),
        _ => panic!(),
    };
    varbuffer.remove(0);
    for exp in varbuffer {
        // println!("{:?}", exp);
        match exp {
            PreExpression::Variable(ref var) => {
                evaluated = match var {
                    Variable::Str(ref string) => Variable::Str(format!("{}", evaluated) + string),
                    Variable::Int(ref int) => match evaluated {
                        Variable::Int(ref computed) => Variable::Int(computed + int),
                        _ => Variable::Str(format!("{}", evaluated) + &int.to_string()),
                    },
                    Variable::Bool(ref bool) => {
                        Variable::Str(format!("{}", evaluated) + &bool.to_string())
                    }
                    Variable::None => evaluated,
                }
            }
            PreExpression::PLUS => {}
            PreExpression::MINUS => {}
            PreExpression::MULTIPLY => {}
            PreExpression::DIVIDE => {}
        }
    }
    // println!("{:?}", evaluated);
    evaluated
}
fn throw(err: &str) {
    println!("error occured when parsing {}", err)
}
#[derive(Debug, Clone)]
enum PreExpression {
    Variable(Variable),
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
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
            Variable::None => write!(f, "{}", ""),
        }
    }
}
impl std::fmt::Display for PreExpression {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match *self {
            PreExpression::Variable(ref var) => write!(f, "{}", var),
            _ => write!(f, "{:?}", *self),
        }
    }
}
