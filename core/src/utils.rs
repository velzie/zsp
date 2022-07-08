#[macro_export]
macro_rules! func {
    ($name:expr,$fn:expr,$args:expr) => {
        (
            String::from($name),
            RFunction {
                func: FunctionType::ExternalFunction($fn),
                args: vec!["".into(); $args],
            },
        )
    };
}
#[macro_export]
macro_rules! afunc {
    ($name:expr,$fn:expr,$args:expr) => {
        (
            String::from($name),
            Rc::new(RefCell::new(Value::Lambda {
                takeself: true,
                func: RFunction {
                    func: FunctionType::ExternalFunction($fn),
                    args: vec!["".into(); $args],
                },
            })),
        )
    };
}