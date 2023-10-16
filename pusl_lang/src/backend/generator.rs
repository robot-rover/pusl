use crate::backend::object::Value::Boolean;
use crate::backend::object::{NativeFn, NativeFnHandle, Object, Value};
use crate::backend::{execute, ExecuteReturn};
use garbage::MarkTrace;
use std::any::Any;
use std::{cell::RefCell, collections::HashMap};

use crate::backend::ExecuteReturn::Yield;
use anymap::AnyMap;
use std::fmt::{Debug, Formatter};

use super::argparse;
use super::object::ObjectPtr;
use super::ExecutionState;
use super::StackFrame;

struct Generator {
    stack: Option<StackFrame>,
    next_val: Option<Value>,
    fn_table: GeneratorBuiltin,
}

impl MarkTrace for Generator {
    fn mark_trace(&self) {
        // TODO:
        // self.stack.map(|stack| stack)
        if let Some(val) = &self.next_val {
            val.mark_trace()
        }
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generator")
            .field("stack", &self.stack)
            .field("next_val", &self.next_val)
            .finish_non_exhaustive()
    }
}

impl Object for Generator {
    fn assign_field(&mut self, _name: &str, _value: Value, _is_let: bool) {
        panic!("Cannot Assign to generator primitive")
    }

    fn get_field(&self, name: &str) -> Value {
        match name {
            "hasNext" => Value::native_fn_index(self.fn_table.has_next),
            "next" => Value::native_fn_index(self.fn_table.next),
            _ => panic!("Unknown field"),
        }
    }

    impl_native_data!();
}

#[derive(Debug)]
struct IterationEnd;

impl MarkTrace for IterationEnd {
    fn mark_trace(&self) {}
}

impl Object for IterationEnd {
    fn assign_field(&mut self, _name: &str, _value: Value, _is_let: bool) {
        panic!("Cannot Assign to Iteration End primitive")
    }

    fn get_field(&self, name: &str) -> Value {
        panic!("Unknown field: '{}'", name)
    }

    impl_native_data!();
}

#[derive(Copy, Clone)]
struct GeneratorBuiltin {
    has_next: NativeFnHandle,
    next: NativeFnHandle,
}

pub fn register(
    builtins: &mut HashMap<&str, Value>,
    registry: &mut Vec<NativeFn>,
    data_map: &mut AnyMap,
) {
    builtins.insert("is_end", Value::native_fn(is_end, registry));
    data_map.insert::<GeneratorBuiltin>(GeneratorBuiltin {
        has_next: Value::native_fn_handle(has_next, registry),
        next: Value::native_fn_handle(next, registry),
    });
}

pub fn new_generator(stack_frame: StackFrame, st: &mut ExecutionState) -> Value {
    let generator_builtins = *st
        .builtin_data
        .get::<GeneratorBuiltin>()
        .expect("Generator Builtins are not loaded");
    let object = RefCell::new(Generator {
        stack: Some(stack_frame),
        next_val: None,
        fn_table: generator_builtins,
    });
    let gc_ptr = st.gc.place_in_heap(object) as ObjectPtr;

    Value::Object(gc_ptr)
}

fn is_end(args: Vec<Value>, this: Option<Value>, _st: &RefCell<ExecutionState>) -> Value {
    assert!(this.is_none());
    let obj: Value = argparse::parse1(args);
    Boolean(check_is_end(&obj))
}

fn check_is_end(value: &Value) -> bool {
    if let Value::Object(ptr) = value {
        ptr.borrow().get_native_data().is::<IterationEnd>()
    } else {
        false
    }
}

fn assemble_end(st: &RefCell<ExecutionState>) -> Value {
    let object = RefCell::new(IterationEnd);
    let gc_ptr = st.borrow_mut().gc.place_in_heap(object) as ObjectPtr;

    Value::Object(gc_ptr)
}

fn has_next<'a: 'b, 'b>(
    args: Vec<Value>,
    this: Option<Value>,
    st: &'a RefCell<ExecutionState<'b>>,
) -> Value {
    argparse::parse0(args);
    if let Some(Value::Object(obj_ptr)) = &this {
        if let Some(generator) = obj_ptr
            .borrow_mut()
            .get_native_data_mut()
            .downcast_mut::<Generator>()
        {
            let has_next = if let Some(next_val) = &generator.next_val {
                !check_is_end(next_val)
            } else {
                let ex_return = run_frame(
                    generator
                        .stack
                        .as_mut()
                        .expect("No stack in generator object"),
                    st,
                );
                if let Yield(val) = ex_return {
                    generator.next_val = Some(val);
                    true
                } else {
                    generator.next_val = Some(assemble_end(st));
                    false
                }
            };
            Value::Boolean(has_next)
        } else {
            panic!("Object is not a generator");
        }
    } else {
        panic!("this is not an object")
    }
}

fn run_frame<'a: 'b, 'b>(
    frame: &mut StackFrame,
    st: &'a RefCell<ExecutionState<'b>>,
) -> ExecuteReturn {
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

pub fn next<'a>(
    args: Vec<Value>,
    this: Option<Value>,
    st: &'a RefCell<ExecutionState<'a>>,
) -> Value {
    argparse::parse0(args);
    if let Some(Value::Object(obj_ptr)) = &this {
        if let Some(generator) = obj_ptr
            .borrow_mut()
            .get_native_data_mut()
            .downcast_mut::<Generator>()
        {
            if let Some(next_val) = generator.next_val.take() {
                if check_is_end(&next_val) {
                    generator.next_val = Some(assemble_end(st));
                }
                next_val
            } else {
                let ex_return = run_frame(
                    generator
                        .stack
                        .as_mut()
                        .expect("No stack in generator object"),
                    st,
                );
                if let Yield(val) = ex_return {
                    val
                } else {
                    generator.next_val = Some(assemble_end(st));
                    assemble_end(st)
                }
            }
        } else {
            panic!("Object is not a generator");
        }
    } else {
        panic!("this is not an object")
    }
}
