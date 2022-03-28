use crate::backend::object::Value::{Boolean, Null};
use crate::backend::object::{NativeFn, NativeFnHandle, Object, Value};
use crate::backend::{BoundFunction, execute, Variable, VariableStack};
use garbage::GcPointer;
use std::{cell::RefCell, collections::HashMap};
use std::convert::{TryFrom, TryInto};
use typemap::{Key, TypeMap};

use super::argparse;
use super::object::ObjectPtr;
use super::ExecutionState;
use super::StackFrame;

struct GenKey;

impl Key for GenKey {
    type Value = (Option<StackFrame>, Option<Value>);
}

struct GenIterEnd;

impl Key for GenIterEnd {
    type Value = ();
}

#[derive(Copy, Clone)]
struct GeneratorBuiltin {
    has_next: NativeFnHandle,
    next: NativeFnHandle,
}

impl Key for GeneratorBuiltin {
    type Value = GeneratorBuiltin;
}

pub fn register<'a>(builtins: &mut HashMap<&str, Value>, registry: &mut Vec<NativeFn>, data_map: &mut TypeMap) {
    builtins.insert("is_end", Value::native_fn(is_end, registry));
    data_map.insert::<GeneratorBuiltin>(GeneratorBuiltin {
        has_next:  Value::native_fn_handle(has_next, registry),
        next: Value::native_fn_handle(next, registry),
    });
}

pub fn new_generator(stack_frame: StackFrame, st: &RefCell<ExecutionState>) -> Value {
    let object = Object::new();
    {
        let mut borrow = object.borrow_mut();
        borrow.native_data.insert::<GenKey>((Some(stack_frame), None));
        let handles = st
            .borrow()
            .builtin_data
            .get::<GeneratorBuiltin>()
            .expect("Generator Builtin not Initialized")
            .clone();
        //TODO: This should be handled with a super object instead
        borrow.let_field(
            String::from("hasNext"),
            Value::native_fn_index(handles.has_next),
        );
        borrow.let_field(
            String::from("next"),
            Value::native_fn_index(handles.next),
        );
    }
    let gc_ptr = st.borrow().gc.borrow_mut().place_in_heap(object);

    Value::Object(gc_ptr)
}

fn is_end(args: Vec<Value>, this: Option<Value>, _st: &RefCell<ExecutionState>) -> Value {
    assert_eq!(this, Option::<Value>::None);
    let obj: Value = argparse::parse1(args);
    Boolean(check_is_end(&obj))
}

fn check_is_end(value: &Value) -> bool {
    match value {
        Value::Object(ptr) => ptr.borrow().native_data.contains::<GenIterEnd>(),
        _ => false
    }
}

fn create_end(args: Vec<Value>, this: Option<Value>, st: &RefCell<ExecutionState>) -> Value {
    assert_eq!(this, None);
    argparse::parse0(args);
    assemble_end(st)
}

fn assemble_end(st: &RefCell<ExecutionState>) -> Value {
    let object = Object::new();
    object.borrow_mut().native_data.insert::<GenIterEnd>(());
    let gc_ptr = st.borrow().gc.borrow_mut().place_in_heap(object);

    Value::Object(gc_ptr)
}

fn has_next<'a: 'b, 'b>(args: Vec<Value>, this: Option<Value>, st: &'a RefCell<ExecutionState<'b>>) -> Value {
    argparse::parse0(args);
    if let Some(Value::Object(obj_ptr)) = &this {
        if let Some((stack, next_data)) = obj_ptr.borrow_mut().native_data.get_mut::<GenKey>() {
            let data = if let Some(to_return) = next_data {
                to_return.clone()
            } else {
                let (value, did_yield) = run_frame(stack.as_mut().expect("No stack in generator object"), st);
                if !did_yield {
                    *next_data = Some(assemble_end(st));
                    return Value::Boolean(false);
                }
                *next_data = Some(value.clone());
                value
            };
            let end = ObjectPtr::try_from(data).map(|obj_ptr| obj_ptr.borrow().native_data.contains::<GenIterEnd>()).unwrap_or(false);
            Value::Boolean(!end)
        } else {
            panic!("Object is not a generator");
        }
    } else {
        panic!("this is not an object")
    }

}

fn run_frame<'a: 'b, 'b>(frame: &mut StackFrame, st: &'a RefCell<ExecutionState<'b>>) -> (Value, bool) {
    let mut old_stack = Vec::new();
    {
        let mut stb = st.borrow_mut();
        std::mem::swap(frame, &mut stb.current_frame);
        std::mem::swap(&mut old_stack, &mut stb.execution_stack);
    }
    let ret_val = execute(st);
    {
        let mut stb = st.borrow_mut();
        std::mem::swap(frame, &mut stb.current_frame);
        std::mem::swap(&mut old_stack, &mut stb.execution_stack);
    }
    ret_val
}

pub fn next<'a>(args: Vec<Value>, this: Option<Value>, st: &'a RefCell<ExecutionState<'a>>) -> Value {
    argparse::parse0(args);
    if let Some(Value::Object(obj_ptr)) = &this {
        if let Some((stack, next_data)) = obj_ptr.borrow_mut().native_data.get_mut::<GenKey>() {
            let stack = stack.as_mut().expect("No stack in generator object");
            let data = if let Some(to_return) = next_data.take() {
                if check_is_end(&to_return) {
                    *next_data = Some(to_return.clone());
                }
                to_return
            } else {
                let (value, is_yield) = run_frame(stack, st);
                if !is_yield {
                    panic!("Iterator out of elements");
                }
                value
            };
            data
        } else {
            panic!("Object is not a generator");
        }
    } else {
        panic!("this is not an object")
    }

}
