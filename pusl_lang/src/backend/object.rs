use crate::backend::GcPoolRef;
use crate::backend::RFunction;
use bitflags::_core::cell::RefCell;
use bitflags::_core::fmt::Formatter;
use garbage::{GcPointer, MarkTrace};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use typemap::TypeMap;

pub type ObjectPtr = GcPointer<RefCell<Object>>;
pub type NativeFn = fn(Vec<Value>, Option<Value>, GcPoolRef) -> Value;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(GcPointer<String>),
    Function(&'static RFunction),
    Native(NativeFn),
    Object(ObjectPtr),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null")?,
            Value::Boolean(val) => write!(f, "{}", val)?,
            Value::Integer(val) => write!(f, "{}", val)?,
            Value::Float(val) => write!(f, "{}", val)?,
            Value::String(val) => write!(f, "{}", **val)?,
            Value::Function(val) => write!(f, "Function {:p}", (*val) as *const _)?,
            Value::Native(val) => write!(f, "NativeFunc {:p}", *val)?,
            Value::Object(val) => {
                write!(f, "Object ")?;
                (*val).write_addr(f)?;
            }
        }
        Ok(())
    }
}

impl Value {
    pub fn type_string(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Boolean(_) => "Boolean",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Function(_) => "Function",
            Value::Object(_) => "Object",
            Value::Native(_) => "Native Function",
        }
    }
}

impl MarkTrace for Value {
    fn mark_children(&self) {
        match self {
            Value::Object(object) => object.mark_recurse(),
            _ => {}
        }
    }
}

impl MarkTrace for Object {
    fn mark_children(&self) {
        if let Some(super_ptr) = &self.super_ptr {
            super_ptr.mark_recurse();
        }
        self.fields.iter().for_each(|(_, v)| {
            if let Value::Object(ptr) = v {
                ptr.mark_recurse();
            }
        })
    }
}

//Todo: The debug impl really should be custom
pub struct Object {
    super_ptr: Option<ObjectPtr>,
    fields: HashMap<String, Value>,
    pub native_data: TypeMap,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("super_ptr", &self.super_ptr)
            .field("fields", &self.fields)
            .finish()
    }
}

impl Object {
    pub fn new() -> RefCell<Self> {
        let object = Object {
            super_ptr: None,
            fields: HashMap::new(),
            native_data: TypeMap::new(),
        };
        RefCell::new(object)
    }

    pub fn new_with_parent(parent: ObjectPtr) -> RefCell<Self> {
        let object = Object {
            super_ptr: Some(parent),
            fields: HashMap::new(),
            native_data: TypeMap::new(),
        };
        RefCell::new(object)
    }

    pub fn get_field(this: ObjectPtr, name: &str) -> Value {
        let mut object_ptr = Some(this);
        while let Some(object) = object_ptr {
            if let Some(value) = object.borrow().fields.get(name).map(|val| (*val).clone()) {
                return value;
            }
            object_ptr = object.borrow().super_ptr.clone();
        }
        Value::Null
    }

    pub fn let_field(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }

    pub fn assign_field(&mut self, name: &str, value: Value) {
        let entry = self.fields.get_mut(name);
        if let Some(old_value) = entry {
            *old_value = value;
        } else {
            panic!("Cannot assign to non-existent field without let")
        }
    }
}
