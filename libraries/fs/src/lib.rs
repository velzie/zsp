use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;
use zsp_core::exceptions::Exception;
use zsp_core::runtime::{
    downcast_dyn, DynObject, DynObjectContainer, FunctionType, RFunction, Value,
};
use zsp_core::{afunc, func};
#[no_mangle]
pub fn lib() -> HashMap<String, RFunction> {
    HashMap::from([func!("fopen", fopen, 1)])
}

#[no_mangle]
pub fn fopen<'a>(inp: Vec<Value<'a>>) -> Result<Value<'a>, Exception> {
    let pstring = &inp[0].to_string();
    let path = Path::new(pstring);
    match fs::File::options()
        .create(true)
        .write(true)
        .read(true)
        .open(path)
    {
        Ok(f) => Ok(Value::DynObject(DynObjectContainer {
            val: Box::new(FileHandle { handle: f }),
        })),
        Err(e) => Err(Exception::new(
            2,
            "FileOpenException",
            &format!("Error while opening file: {}", e),
        )),
    }
}

#[derive(Debug)]
struct FileHandle {
    pub handle: File,
}
impl Clone for FileHandle {
    fn clone(&self) -> Self {
        panic!("tried to clone a filehandle. this SHOULDNT happen, but it did")
    }
}
impl<'a> DynObject<'a> for FileHandle {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
    fn fields(&self) -> HashMap<String, Rc<RefCell<Value<'a>>>> {
        HashMap::from([
            afunc!("read", handle_read, 0),
            afunc!("write", handle_write, 1),
        ])
    }
}

#[no_mangle]
pub fn handle_read<'a>(mut inp: Vec<Value<'a>>) -> Result<Value<'a>, Exception> {
    let mut handle =
        &downcast_dyn::<FileHandle>(inp[0].as_ref().borrow_mut().as_dyn_object()).handle;

    let mut buf = String::new();
    match handle.read_to_string(&mut buf) {
        Ok(_) => Ok(Value::String(buf)),
        Err(e) => Err(Exception::new(
            2,
            "FileReadException",
            &format!("Error while reading from file: {}", e),
        )),
    }
}

#[no_mangle]
pub fn handle_write<'a>(mut inp: Vec<Value<'a>>) -> Result<Value<'a>, Exception> {
    let mut handle =
        &downcast_dyn::<FileHandle>(inp[0].as_ref().borrow_mut().as_dyn_object()).handle;

    match handle.write_fmt(format_args!("{}", inp[1].to_string())) {
        Ok(_) => Ok(Value::Null),
        Err(e) => Err(Exception::new(
            2,
            "FileWriteException",
            &format!("Error while writing to file: {}", e),
        )),
    }
}
