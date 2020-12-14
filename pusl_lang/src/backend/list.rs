use crate::backend::object::{Object, Value};
use crate::backend::GcPoolRef;
use std::collections::HashMap;
use typemap::Key;

struct ListKey;

impl Key for ListKey {
    type Value = Vec<Value>;
}

pub fn register(builtins: &mut HashMap<&str, Value>) {
    builtins.insert("List", Value::native_fn(new_list));
}

fn new_list(args: Vec<Value>, _: Option<Value>, gc: GcPoolRef) -> Value {
    let object = Object::new();
    {
        let mut borrow = object.borrow_mut();
        borrow.native_data.insert::<ListKey>(args);
        //TODO: This should be handled with a super object instead
        borrow.let_field(String::from("push"), Value::native_fn(list_push));
        borrow.let_field(String::from("@index_get"), Value::native_fn(list_index_get));
        borrow.let_field(String::from("@index_set"), Value::native_fn(list_index_set));
    }
    let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(object));

    Value::Object(gc_ptr)
}

fn get_list_vec<R, T: FnOnce(&mut Vec<Value>) -> R>(object: &Option<Value>, action: T) -> R {
    if let Some(Value::Object(gc_ptr)) = object {
        if let Some(vec) = gc_ptr.borrow_mut().native_data.get_mut::<ListKey>() {
            action(vec)
        } else {
            panic!("Object is not a List")
        }
    } else {
        panic!("Argument is not an Object")
    }
}

fn list_push(mut args: Vec<Value>, this: Option<Value>, _: GcPoolRef) -> Value {
    let value = args.pop().expect("must call push with 1 argument");
    assert!(args.is_empty());
    get_list_vec(&this, |vec| vec.push(value));
    Value::Null
}

fn list_index_get(mut args: Vec<Value>, this: Option<Value>, _: GcPoolRef) -> Value {
    let index = args.pop().expect("must call @index_get with 1 argument");
    assert!(args.is_empty());
    let index = if let Value::Integer(index) = index {
        index as usize
    } else {
        panic!("Can only index list with integer")
    };
    let element = get_list_vec(&this, |vec| vec.get(index).cloned());
    element.expect("Index out of bounds")
}

fn list_index_set(mut args: Vec<Value>, this: Option<Value>, _: GcPoolRef) -> Value {
    let index: Value = args.pop().expect("must call @index_set with 2 arguments");
    let value: Value = args.pop().expect("must call @index_set with 2 arguments");
    let index = if let Value::Integer(index) = index {
        index as usize
    } else {
        panic!("Can only index list with integer")
    };
    get_list_vec(&this, |vec| {
        let reference = vec.get_mut(index).expect("Index out of bounds");
        *reference = value;
    });

    Value::Null
}
