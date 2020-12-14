use crate::backend::object::{Object, Value};
use std::collections::HashMap;
use typemap::Key;

use super::{
    argparse,
    object::{FunctionTarget, MethodPtr, ObjectPtr},
    ExecutionState, StackFrame,
};

struct GenKey<'a>;

impl<'a> Key for GenKey<'a> {
    type Value = Option<StackFrame<'a>>;
}

struct EndKey;

impl Key for EndKey {
    type Value = ();
}

pub fn register<'a>(builtins: &mut HashMap<&str, Value<'a>>) {
    builtins.insert("is_end", Value::native_fn(is_end));
}

fn is_end<'a>(args: Vec<Value>, _: Option<Value>, st: &RefCell<ExecutionState>) -> Value<'a> {
    let gc_ptr: ObjectPtr = argparse::parse1(args);
    let state = gc_ptr.borrow_mut().native_data.contains::<EndKey>();
    Value::Boolean(state)
}

fn new_generator<'a>(args: Vec<Value>, function: Value, st: &RefCell<ExecutionState>) -> Value<'a> {
    let (func, this) = match argparse::convert_arg::<MethodPtr>(function, 1) {
        (FunctionTarget::Pusl(func), this) => (func, this),
        (FunctionTarget::Native(func), _) => {
            panic!("Cannot make generator from native function") //TODO: Better Debug
        }
    };
    let frame = StackFrame::from_function(func, this, args);
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
