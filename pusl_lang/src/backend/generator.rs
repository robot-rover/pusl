use crate::backend::object::{Object, Value};
use std::{cell::RefCell, collections::HashMap};
use typemap::Key;

use super::{
    argparse,
    object::{FnPtr, FunctionTarget, MethodPtr, ObjectPtr},
    ExecutionState, StackFrame,
};

struct GenKey;

impl Key for GenKey {
    type Value = (Option<StackFrame>, Option<Value>);
}

pub fn register<'a>(builtins: &mut HashMap<&str, Value>) {
    builtins.insert("is_end", Value::native_fn(is_end, builtins));
}

pub fn has_next<'a>(args: Vec<Value>, this: Option<Value>, st: &'a RefCell<ExecutionState>) -> Value {

}

pub fn next<'a>(args: Vec<Value>, this: Option<Value>, st: &'a RefCell<ExecutionState>) -> Value {

}

fn new_generator<'a>(
    args: Vec<Value>,
    function: MethodPtr,
    this: Option<Value>,
    st: &'a RefCell<ExecutionState>,
) -> Value {
    let frame = StackFrame::from_function(function, this, args);
    let object = Object::new();
    {
        let mut borrow = object.borrow_mut();
        borrow.native_data.insert::<GenKey>(Some(frame));
    }
    let gc_ptr = st.gc.borrow_mut().place_in_heap(object);

    Value::Object(gc_ptr)
}

fn get_frame(generator: Value) -> StackFrame {
    let gc_ptr: ObjectPtr = argparse::convert_arg(generator, 0);
    let frame = if let Some(frame_holder) = gc_ptr.borrow_mut().native_data.get_mut::<GenKey>() {
        frame_holder
            .take()
            .expect("generator frame is already taken")
    } else {
        panic!("object is not a generator")
    };
    frame
}

fn set_frame(generator: Value, frame: StackFrame) {
    let gc_ptr: ObjectPtr = argparse::convert_arg(generator, 0);
    if let Some(frame_holder) = gc_ptr.borrow_mut().native_data.get_mut::<GenKey>() {
        assert!(
            frame_holder.replace(frame).is_none(),
            "generator frame already exists"
        );
    };
}
