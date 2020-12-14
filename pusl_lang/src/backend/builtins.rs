use super::object::ObjectPtr;
use crate::backend::argparse;
use crate::backend::list;
use crate::backend::object::{Object, Value};
use crate::backend::GcPoolRef;
use std::collections::HashMap;

pub fn get_builtins() -> HashMap<&'static str, Value> {
    let mut map = HashMap::new();
    map.insert("type_of", Value::native_fn(type_of));
    map.insert("print", Value::native_fn(print));
    map.insert("native", Value::native_fn(native_import));
    map.insert("Object", Value::native_fn(new_object));

    list::register(&mut map);

    map
}

fn type_of(args: Vec<Value>, _: Option<Value>, gc: GcPoolRef) -> Value {
    let value: Value = argparse::parse1(args);
    let type_string = value.type_string();
    let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(type_string.to_owned()));
    Value::String(gc_ptr)
}

fn print(args: Vec<Value>, _: Option<Value>, _: GcPoolRef) -> Value {
    for value in args.into_iter() {
        print!("{}", value);
    }
    Value::Null
}

fn native_import(args: Vec<Value>, _: Option<Value>, _: GcPoolRef) -> Value {
    #[allow(unused_variables)]
    let import_name: Value = argparse::parse1(args);
    unimplemented!();
}

fn new_object(args: Vec<Value>, _: Option<Value>, gc: GcPoolRef) -> Value {
    let super_obj: Option<ObjectPtr> = argparse::parse_option(args);

    let object_ptr = if let Some(super_obj) = super_obj {
        Object::new_with_parent(super_obj)
    } else {
        Object::new()
    };
    let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(object_ptr));

    Value::Object(gc_ptr)
}
