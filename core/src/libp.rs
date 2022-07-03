use core::panic;
use std::collections::HashMap;
use std::fs;
use std::path;
pub fn load_lib(libname: String) -> Library {
    println!("libname requested is {}", libname);

    dbg!(&libname);
    let rawfs =
        fs::read_to_string(&libname).expect(&format!("could not locate library {}", &libname));
    match json::parse(&rawfs) {
        Ok(json) => {
            let static_loc = &json["static_loc"].to_string();
            if path::Path::new(static_loc).exists() {
                let rawbinds = &json["binds"];
                let mut binds = HashMap::new();
                rawbinds.entries().for_each(|f| {
                    binds.insert(
                        f.0.to_string(),
                        Bind {
                            name: f.0.to_string(),
                            args: f.1["args"].members().map(|f| f.to_string()).collect(),
                            bound_symbol: f.1["bound_symbol"].to_string(),
                        },
                    );
                });
                Library {
                    name: json["name"].to_string(),
                    static_loc: static_loc.clone(),
                    binds,
                }
            } else {
                panic!(
                    "could not find static object {} requested in library {}",
                    static_loc, libname
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}
#[derive(Debug, Clone)]
pub struct Library {
    pub name: String,
    pub static_loc: String,
    pub binds: HashMap<String, Bind>,
}
#[derive(Debug, Clone)]
pub struct Bind {
    pub name: String,
    pub args: Vec<String>,
    pub bound_symbol: String,
}
