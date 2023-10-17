use super::{
    object::{NativeFn, ObjectPtr},
    ExecutionState,
};
use crate::backend::list;
use crate::backend::object::{PuslObject, Value};
use crate::backend::{argparse, generator};
use anymap::AnyMap;
use std::{cell::RefCell, collections::HashMap};

pub fn get_builtins(registry: &mut Vec<NativeFn>) -> (HashMap<&'static str, Value>, AnyMap) {
    let mut map = HashMap::new();
    let mut data_map = AnyMap::new();
    map.insert("type_of", Value::native_fn(type_of, registry));
    map.insert("print", Value::native_fn(print, registry));
    map.insert("println", Value::native_fn(println, registry));
    map.insert("native", Value::native_fn(native_import, registry));
    map.insert("Object", Value::native_fn(new_object, registry));

    list::register(&mut map, registry, &mut data_map);
    generator::register(&mut map, registry, &mut data_map);

    (map, data_map)
}

fn type_of(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    let value: Value = argparse::parse1(args);
    let type_string = value.type_string();
    let gc_ptr = st.borrow_mut().gc.place_in_heap(type_string.to_owned());
    Value::String(gc_ptr)
}

fn print(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    for value in args.into_iter() {
        write!(st.borrow_mut().stream, "{}", value).unwrap();
    }
    Value::Null
}

fn println(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    for value in args.into_iter() {
        write!(st.borrow_mut().stream, "{}", value).unwrap();
    }
    write!(st.borrow_mut().stream, "\n").unwrap();

    Value::Null
}

fn native_import(args: Vec<Value>, _: Option<Value>, _: &RefCell<ExecutionState>) -> Value {
    #[allow(unused_variables)]
    let import_name: Value = argparse::parse1(args);
    unimplemented!();
}

fn new_object(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    let super_obj: Option<ObjectPtr> = argparse::parse_option(args);

    let object_ptr = if let Some(super_obj) = super_obj {
        PuslObject::new_with_parent(super_obj)
    } else {
        PuslObject::new()
    };
    let gc_ptr = st.borrow_mut().gc.place_in_heap(object_ptr) as ObjectPtr;

    Value::Object(gc_ptr)
}
