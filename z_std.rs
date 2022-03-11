use ::std::collections::HashMap;

pub struct std;

pub struct Func {}

impl std {
    pub fn get(&self) -> HashMap<&str, Func> {
        let mut m: HashMap<&str, Func> = HashMap::new();
        m.insert("print", Func {});
        m
    }
    pub fn print(&self, text: &str) {
        println!("{}", text);
    }
}
