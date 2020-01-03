use garbage::{MarkTrace, GcPointer};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::parser::Expression;
use core::borrow::Borrow;

enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(GcPointer<RefCell<String>>),
    Function(Vec<String>, Vec<Box<dyn Expression>>),
    Object(HashMap<String, GcPointer<RefCell<Value>>>)
}

impl ValueCast for Value {
    fn as_boolean(&self) -> bool {
        if let Value::Boolean(val) = self {
            return *val
        }
        panic!("Value was not boolean: {}", self.type_string())
    }

    fn type_string(&self) -> &str {
        match self {
            Value::Null => "Null",
            Value::Boolean(_) => "Boolean",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Function(_, _) => "Function",
            Value::Object(_) => "Object"
        }
    }
}

impl ValueCast for Option<Value> {
    fn as_boolean(&self) -> bool {
        if let Some(Value::Boolean(val)) = self {
            return *val
        }
        panic!("Value was not boolean: {}", self.type_string())
    }

    fn type_string(&self) -> &str {
        if let Some(value) = self {
            value.type_string()
        } else {
            "Undefined"
        }
    }
}

impl MarkTrace for Value {
    fn mark_children(&self) {
        match self {
            Value::Object(properties) => properties.values().for_each(|v| v.borrow().borrow().mark_recurse()),
            _ => {}
        }
    }
}

trait ValueCast {
    fn as_boolean(&self) -> bool;
    fn type_string(&self) -> &str;
}