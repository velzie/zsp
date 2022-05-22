use std::{collections::HashMap, hash::Hash, ptr::NonNull, rc::Rc};

use crate::parser::{Block, ExpressionFragment, Fragment, Function, Root};
pub fn interpret(root: Root) {
    // dbg!(root);
    // root.root.Run();
    root.root.to_scope().execute(&root.functions);
}

impl Block {
    pub fn to_scope(&self) -> Scope {
        Scope {
            variables: HashMap::new(),
            structure: self.clone(),
        }
    }
}
impl Scope {
    pub fn execute(&mut self, functions: &HashMap<String, Function>) -> Option<Value> {
        for fragment in &self.structure.children {
            match fragment {
                Fragment::If {
                    trueblock,
                    falseblock,
                    condition,
                } => {
                    if evaluate_expression(&condition).to_bool() {
                        trueblock.to_scope().execute(&functions);
                    } else {
                        match falseblock {
                            Some(b) => b.to_scope().execute(&functions),
                            None => None,
                        };
                    }
                }
                Fragment::Assignment { name, value } => {
                    self.variables
                        .insert(name.clone(), Rc::new(evaluate_expression(value)));
                }
                Fragment::InvokeExpression(exp) => {
                    match &exp {
                        ExpressionFragment::Call { name, args } => {
                            functions
                                .get(name)
                                .unwrap()
                                .source
                                .to_scope()
                                .execute(&functions);
                        }
                        _ => {
                            // do later
                            panic!();
                        }
                    }
                }
            }
        }
        None
    }
}

fn evaluate_expression(expression: &Vec<ExpressionFragment>) -> Value {
    Value::Bool(false)
}
pub struct Scope {
    variables: HashMap<String, Rc<Value>>,
    structure: Block,
}

pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}
impl Value {
    pub fn to_bool(&self) -> bool {
        match &self {
            Self::Bool(b) => return *b,
            _ => panic!(),
        }
    }
}
