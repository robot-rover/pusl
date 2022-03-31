use crate::backend::object::{NativeFnHandle, Object, ObjectPtr, Value};
use std::{cell::RefCell, collections::HashMap, fmt};
use std::any::Any;
use std::fmt::{Debug, Formatter};
use anymap::AnyMap;
use garbage::MarkTrace;
use crate::backend::argparse;

use super::{
    object::{NativeFn},
    ExecutionState,
};

struct List { vec: Vec<Value>, fn_table: ListBuiltin }

impl Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("List")
            .field("vec", &self.vec)
            .finish_non_exhaustive()
    }
}

impl MarkTrace for List {
    fn mark_children(&self) {
        for value in self.vec.iter() {
            value.mark_children()
        }
    }
}

impl Object for List {
    fn assign_field(&mut self, name: &str, value: Value, is_let: bool) {
        panic!("Cannot Assign to list primitive")
    }

    fn get_field(&self, name: &str) -> Value {
        let list_builtin = &self.fn_table;
        match name {
            "push" => Value::native_fn_index(list_builtin.push_index),
            "len" => Value::native_fn_index(list_builtin.list_len),
            "@index_get" => Value::native_fn_index(list_builtin.list_index_get),
            "@index_set" => Value::native_fn_index(list_builtin.list_index_set),
            _ => panic!("Unknown field"),
        }
    }

    impl_native_data!();
}


#[derive(Copy, Clone)]
struct ListBuiltin {
    push_index: NativeFnHandle,
    list_index_get: NativeFnHandle,
    list_index_set: NativeFnHandle,
    list_len: NativeFnHandle,
}

pub fn register(
    builtins: &mut HashMap<&str, Value>,
    registry: &mut Vec<NativeFn>,
    data_map: &mut AnyMap,
) {
    builtins.insert("List", Value::native_fn(new_list, registry));
    data_map.insert::<ListBuiltin>(ListBuiltin {
        push_index: Value::native_fn_handle(list_push, registry),
        list_index_get: Value::native_fn_handle(list_index_get, registry),
        list_index_set: Value::native_fn_handle(list_index_set, registry),
        list_len: Value::native_fn_handle(list_len, registry),
    });
}

fn new_list(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    let list_builtins = *st.borrow().builtin_data.get::<ListBuiltin>().expect("List Builtins are not loaded");
    let object = RefCell::new(List { vec: args, fn_table: list_builtins});

    let gc_ptr = st.borrow().gc.borrow_mut().place_in_heap(object) as ObjectPtr;

    Value::Object(gc_ptr)
}

fn get_list_vec<R, T: FnOnce(&mut Vec<Value>) -> R>(object: &Option<Value>, action: T) -> R {
    if let Some(Value::Object(gc_ptr)) = object {
        let mut gc_borrow = gc_ptr.borrow_mut();
        let type_data = gc_borrow.get_native_data_mut().downcast_mut::<List>();
        if let Some(vec) = type_data {
            action(&mut vec.vec)
        } else {
            panic!("Object is not a List: {:?}", gc_borrow)
        }
    } else {
        panic!("Argument is not an Object")
    }
}

fn list_push(mut args: Vec<Value>, this: Option<Value>, _: &RefCell<ExecutionState>) -> Value {
    let value = args.pop().expect("must call push with 1 argument");
    assert!(args.is_empty());
    get_list_vec(&this, |vec| vec.push(value));
    Value::Null
}

fn list_len(args: Vec<Value>, this: Option<Value>, _: &RefCell<ExecutionState>) -> Value {
    argparse::parse0(args);
    let len = get_list_vec(&this, |vec| vec.len());
    Value::Integer(len as i64)
}

fn list_index_get(mut args: Vec<Value>, this: Option<Value>, _: &RefCell<ExecutionState>) -> Value {
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

fn list_index_set(mut args: Vec<Value>, this: Option<Value>, _: &RefCell<ExecutionState>) -> Value {
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
