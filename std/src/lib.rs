use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use zsp_core::runtime::{DynObject, DynObjectContainer, Value};
#[no_mangle]
pub fn celesteobj<'a>(inp: Vec<Value<'a>>) -> Value<'a> {
    Value::DynObject(DynObjectContainer {
        val: Box::new(Celeste {
            mem: "adsad".into(),
            val: 1,
        }),
    })
}

#[derive(Debug, Clone)]
struct Celeste {
    pub mem: String,
    pub val: i32,
}
impl<'a> DynObject<'a> for Celeste {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
    fn fields(&self) -> HashMap<String, Rc<RefCell<Value<'a>>>> {
        HashMap::new()
    }
}
