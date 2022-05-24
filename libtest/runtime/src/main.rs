// extern crate liblib;
use libloading::Library;
fn main() {
    println!("Hello, world!");

    unsafe {
        let lib = Library::new("/home/ce/Documents/GitHub/zsp/libtest/runtime/liblib.so").unwrap();
        let fnc: libloading::Symbol<unsafe extern "C" fn(Vec<Const>) -> Vec<Const>> =
            lib.get(b"callable_from_c").unwrap();
        dbg!(fnc(vec![Const::String("h".to_string())]));
        // println!("{}", fnc());
    }
    // liblib::callable_from_c(3);
    // unsafe {
    //     callable_from_c(4);
    // }
}

#[derive(Debug)]
#[repr(C)]
pub enum Const {
    String(String),
    u32(u32),
}

// #[link(name = "liblib")]
// extern "C" {
//     fn callable_from_c(x: i32) -> bool;
// }
