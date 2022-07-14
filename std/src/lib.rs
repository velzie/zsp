use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use zsp_core::exceptions::Exception;
use zsp_core::func;
use zsp_core::runtime::{DynObject, DynObjectContainer, FunctionType, RFunction, Value};
#[no_mangle]
pub fn lib() -> HashMap<String, RFunction> {
    HashMap::from([func!("celeste", celesteobj, 0)])
}

#[no_mangle]
pub fn celesteobj<'a>(inp: Vec<Value<'a>>) -> Result<Value<'a>, Exception> {
    Ok(Value::DynObject(DynObjectContainer {
        val: Box::new(Celeste {
            mem: "adsad".into(),
            val: 1,
        }),
    }))
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
