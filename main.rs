#[macro_use]
extern crate lazy_static;
#[allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;
use substring::Substring;
// mod zstd;
// use zstd::Zstd;

lazy_static! {
    static ref VARIABLES: Mutex<HashMap<String, Variable>> = {
        let mut m = HashMap::new();
        m.insert(String::from("true"), Variable::Bool(true));
        m.insert(String::from("false"), Variable::Bool(false));
        Mutex::new(m)
    };
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let contents = fs::read_to_string(&args[1]).expect("could not read file");
        interpret(&contents);
        // Zstd.std
        // let mut fnmap: HashMap<&str, Box<dyn Fn(_)>> = HashMap::new();
        // fnmap.insert(
        //     "func",
        //     Box::new(|x: &str| println!("{}", x)) as Box<dyn Fn(_)>,
        // );
        // fnmap.get("func").unwrap()("e");
        // let fnc: closure = |e: &str| print!("{}", e);
        // print!("{}", std::any::type_name(fnc));
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
    // let top: Vec<&str> = line.split(" ").collect();
    // let word: &str = top[0];
    // let vars = VARIABLES.lock().unwrap();
    // println!("{}", word);
    // match word {
    // _ => {
    // if true {
    // drop(vars);
    return consume_std(line);
    // } else if vars.contains_key(word) {
    // drop(vars);
    // }
    // println!("{} is a undefined keyword", word);
    // return "".to_string();
    // panic!("{} is a undefined keyword", word);
    // }
    // }
}
fn consume_std(code: &str) -> String {
    //example input
    // add 1,2
    // print x
    // print "goodbye"

    let code: (String, String) = split_string(code, "\n");
    // add 1,2
    let kargs = split_string(&code.0, " ");
    // (add|1,2)
    let args: Vec<String> = split_expression(kargs.1, ',');
    // println!();
    // [1,2
    match kargs.0.as_str() {
        "print" => println!("from code: {}", evaluate_expression(&args[0])),
        "if" => {
            let block = split_bool(&code.1);
            match evaluate_expression(&args[0]) {
                Variable::Bool(ref res) => {
                    if *res {
                        return consume_std(&block.0);
                    } else {
                        println!("\n{}\n", block.1);
                        if block.1 != "" {
                            return consume_std(&block.1);
                        }
                    }
                }
                _ => panic!("expected bool, got something else"),
            }
            return block.2;
        }
        _ => println!("keyword {} is not defined at line {}", kargs.0, code.0),
    }
    return code.1;
}
fn split_string(input: &str, character: &str) -> (String, String) {
    match input.find(character) {
        Some(index) => (
            input.substring(0, index).to_string(),
            input
                .substring(index + character.len(), input.len())
                .to_string(),
        ),
        None => (input.to_string(), String::default()),
    }
}

fn split_bool(input: &str) -> (String, String, String) {
    let ifs: Vec<(usize, &str)> = input.match_indices("if").collect();
    let elses: Vec<(usize, &str)> = input.match_indices("else").collect();
    let fis: Vec<(usize, &str)> = input.match_indices("fi").collect();

    let mut ifc = String::default();
    let mut elsec = String::default();
    let mut remc = String::default();
    let mut innerifindex = 0;
    let mut inelse = false;
    let mut elseindx = 0;
    let mut instring = false;
    for i in 0..input.len() {
        if input.chars().nth(i).unwrap() == '"' {
            instring = !instring;
        }
        if ifs.contains(&(i, "if")) && !instring {
            innerifindex += 1;
        }
        if elses.contains(&(i, "else")) && !instring {
            if innerifindex == 0 {
                elseindx = i;
                inelse = true;
                ifc = input.substring(0, i).to_string();
            }
        }
        if fis.contains(&(i, "fi")) && !instring {
            if innerifindex == 0 {
                if inelse {
                    elsec = input.substring(elseindx + 4, i).to_string();
                } else {
                    ifc = input.substring(0, i).to_string();
                }
                remc = input.substring(i, input.len()).to_string();
            } else {
                innerifindex -= 1;
            }
        }
    }
    // println!("ifc [{}] elsec [{}]", ifc, elsec);
    return (ifc, elsec, remc);

    // match input.find(character) {
    //     Some(index) => (
    //         input.substring(0, index).to_string(),
    //         input
    //             .substring(index + character.len(), input.len())
    //             .to_string(),
    //     ),
    //     None => (input.to_string(), String::default()),
    // }
}

fn evaluate_expression(expression: &str) -> Variable {
    println!("evaluating the expression {}", expression);
    let args: Vec<String> = split_expression(expression.to_string(), ' ');
    let vars = VARIABLES.lock().unwrap();

    let mut varbuffer: Vec<PreExpression> = Vec::new();
    for arg in args {
        if vars.contains_key(&arg) {
            varbuffer.push(PreExpression::Variable(vars.get(&arg).unwrap().clone()));
        } else if match arg.parse::<i32>() {
            Ok(n) => {
                varbuffer.push(PreExpression::Variable(Variable::Int(n)));
                true
            }
            Err(_) => false,
        } {
        } else if match &*arg {
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
            "==" => {
                varbuffer.push(PreExpression::EQUALS);
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
    let mut held: Option<Variable> = None;
    let mut operation: Option<PreExpression> = None;
    for exp in varbuffer {
        match exp {
            PreExpression::Variable(ref var) => match held.clone() {
                Some(heldvar) => match operation.clone() {
                    Some(op) => {
                        held = Some(match op {
                            PreExpression::PLUS => match var {
                                Variable::Str(ref string) => {
                                    Variable::Str(format!("{}", heldvar) + string)
                                }
                                Variable::Int(ref int) => match heldvar {
                                    Variable::Int(ref computed) => Variable::Int(computed + int),
                                    _ => Variable::Str(format!("{}", heldvar) + &int.to_string()),
                                },
                                Variable::Bool(ref bool) => {
                                    Variable::Str(format!("{}", heldvar) + &bool.to_string())
                                }
                                Variable::None => heldvar,
                            },
                            PreExpression::MINUS => match var {
                                Variable::Int(ref int) => match heldvar {
                                    Variable::Int(ref computed) => Variable::Int(computed - int),
                                    _ => panic!("cant subtract that"),
                                },
                                _ => panic!("cant subtract that"),
                            },
                            PreExpression::MULTIPLY => match var {
                                Variable::Int(ref int) => match heldvar {
                                    Variable::Int(ref computed) => Variable::Int(computed * int),
                                    _ => panic!("cant multiply that"),
                                },
                                _ => panic!("cant multiply that"),
                            },
                            PreExpression::DIVIDE => match var {
                                Variable::Int(ref int) => match heldvar {
                                    Variable::Int(ref computed) => Variable::Int(computed / int),
                                    _ => panic!("cant divide that"),
                                },
                                _ => panic!("cant divide that"),
                            },
                            PreExpression::EQUALS => match var {
                                Variable::Bool(ref bool) => match heldvar {
                                    Variable::Bool(ref computed) => {
                                        Variable::Bool(computed == bool)
                                    }
                                    _ => panic!("thats not a bool"),
                                },
                                _ => panic!("thats not a bool"),
                            },
                            _ => panic!("you need an operator. that doesn't make any sense"),
                        });
                        operation = None;
                    }
                    None => panic!("specify an operator in expression"),
                },
                None => {
                    held = Some(match exp.clone() {
                        PreExpression::Variable(ref var) => var.clone(),
                        _ => panic!(),
                    })
                }
            },
            _ => operation = Some(exp),
        }
    }
    held.unwrap()
}
fn split_expression(kargs: String, splitchar: char) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    let mut instr = false;
    let mut buffer = String::default();
    for chr in kargs.chars() {
        if chr == '"' {
            instr = !instr;
            buffer += &chr.to_string();
        } else if chr == splitchar {
            if !instr {
                args.push(buffer);
                buffer = String::default();
            } else {
                buffer += &chr.to_string();
            }
        } else {
            buffer += &chr.to_string();
        }
    }
    // println!("buffe is {}", buffer);
    args.push(buffer);
    args
}
fn _panic(err: &str) {
    throw(err);
    panic!();
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
    EQUALS,
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PreExpression::Variable(ref var) => write!(f, "{}", var),
            _ => write!(f, "{:?}", *self),
        }
    }
}
