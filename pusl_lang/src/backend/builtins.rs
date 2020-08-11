use crate::backend::list;
use crate::backend::object::{Object, Value};
use crate::backend::GcPoolRef;
use std::collections::HashMap;

pub fn get_builtins() -> HashMap<&'static str, Value> {
    let mut map = HashMap::new();
    map.insert("type_of", Value::Native(type_of));
    map.insert("print", Value::Native(print));
    map.insert("native", Value::Native(native_import));
    map.insert("Object", Value::Native(new_object));

    list::register(&mut map);

    map
}

fn type_of(mut args: Vec<Value>, _: Option<Value>, gc: GcPoolRef) -> Value {
    if let Some(value) = args.pop() {
        if !args.is_empty() {
            panic!()
        }
        let type_string = value.type_string();
        let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(type_string.to_owned()));
        Value::String(gc_ptr)
    } else {
        panic!()
    }
}

fn print(args: Vec<Value>, _: Option<Value>, _: GcPoolRef) -> Value {
    for value in args.into_iter().rev() {
        print!("{}", value);
    }
    Value::Null
}

fn native_import(mut args: Vec<Value>, _: Option<Value>, _: GcPoolRef) -> Value {
    let import_name = args.pop().expect("native takes 1 argument");
    assert!(args.is_empty());
    unimplemented!();
}

fn new_object(mut args: Vec<Value>, _: Option<Value>, gc: GcPoolRef) -> Value {
    if args.len() > 1 {
        panic!()
    }
    let object = if let Some(super_obj) = args.pop() {
        let super_obj = if let Value::Object(ptr) = super_obj {
            ptr
        } else {
            panic!()
        };
        Object::new_with_parent(super_obj)
    } else {
        Object::new()
    };
    let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(object));

    Value::Object(gc_ptr)
}
