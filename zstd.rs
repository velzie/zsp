// use std::collections::HashMap;
// type Callback<'a> = Box<(Fn<(&'a mut State,), Output = ()> + 'static)>;
// pub struct Zstd;

// pub struct Func {}a

// lazy_static! {
//     static ref STD<T>: HashMap<&'static str, Box<dyn Fn(T)>> = {
//         let m = HashMap::new();
//         m
//     };
// }
// impl Zstd {
//     pub fn std(&self) -> HashMap<&str, Box<dyn Fn(_)>> {
//         let mut m: HashMap<&str, Func> = HashMap::new();
//         m.insert("print", Func {});
//         m
//     }
// }
// fn print(text: &str) {
//     println!("{}", text);
// }
