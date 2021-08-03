use super::{BoundFunction, ExecutionState, StackFrame};
use bitflags::_core::cell::RefCell;
use bitflags::_core::fmt::Formatter;
use fmt::Display;
use garbage::{GcPointer, MarkTrace};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use typemap::TypeMap;

pub type ObjectPtr = GcPointer<RefCell<Object>>;
pub type StringPtr = GcPointer<String>;
pub type NativeFn<'a> = fn(Vec<Value>, Option<Value>, &'a RefCell<ExecutionState>) -> Value;
pub type FnPtr = GcPointer<BoundFunction>;
pub type GeneratorFn = GcPointer<StackFrame>;
pub type MethodPtr = (FunctionTarget, Option<ObjectPtr>);

pub type NativeFnHandle = usize;

#[derive(Clone, Debug)]
pub enum FunctionTarget {
    Native(NativeFnHandle),
    Pusl(FnPtr),
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(StringPtr),
    Function(MethodPtr),
    Object(ObjectPtr),
}

macro_rules! value_try_from {
    ($datatype:ty, $enumval:path) => {
        impl TryFrom<Value> for $datatype {
            type Error = &'static str;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                if let $enumval(value) = value {
                    Ok(value)
                } else {
                    Err(concat!("Value is not a ", stringify!($enumval)))
                }
            }
        }

        impl TryFrom<Value> for Option<$datatype> {
            type Error = &'static str;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Null => Ok(None),
                    $enumval(value) => Ok(Some(value)),
                    _ => Err(concat!("Value is not a ", stringify!($enumval))),
                }
            }
        }
    };
}

value_try_from!(bool, Value::Boolean);
value_try_from!(i64, Value::Integer);
value_try_from!(f64, Value::Float);
value_try_from!(StringPtr, Value::String);
value_try_from!(MethodPtr, Value::Function);
value_try_from!(ObjectPtr, Value::Object);

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null")?,
            Value::Boolean(val) => write!(f, "{}", val)?,
            Value::Integer(val) => write!(f, "{}", val)?,
            Value::Float(val) => write!(f, "{}", val)?,
            Value::String(val) => write!(f, "{}", **val)?,
            Value::Function((FunctionTarget::Pusl(val), Some(this))) => {
                write!(f, "Bound Function {:?} @ {:?}", val, this)?
            }
            Value::Function((FunctionTarget::Pusl(val), None)) => write!(f, "Function {:?}", val)?,
            Value::Function((FunctionTarget::Native(val), Some(this))) => {
                write!(f, "Bound NativeFunc {} @ {:?}", *val, this)?
            }
            Value::Function((FunctionTarget::Native(val), None)) => {
                write!(f, "NativeFunc {}", *val)?
            }
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
            Value::Function((FunctionTarget::Pusl(_), _)) => "Pusl Function",
            Value::Function((FunctionTarget::Native(_), _)) => "Native Function",
            Value::Object(_) => "Object",
        }
    }

    pub fn native_fn<'a>(function: NativeFn<'a>, registry: &mut Vec<NativeFn<'a>>) -> Self {
        let index = registry.len();
        registry.push(function);
        Value::Function((FunctionTarget::Native(index), None))
    }

    pub fn native_fn_handle<'a>(
        function: NativeFn<'a>,
        registry: &mut Vec<NativeFn<'a>>,
    ) -> NativeFnHandle {
        let index = registry.len();
        registry.push(function);
        index
    }

    pub fn native_fn_index(handle: NativeFnHandle) -> Self {
        Value::Function((FunctionTarget::Native(handle), None))
    }

    pub fn pusl_fn(function: FnPtr) -> Self {
        Value::Function((FunctionTarget::Pusl(function), None))
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

    pub fn get_field(object: &ObjectPtr, name: &str) -> Value {
        //TODO: Bad Recursion
        if let Some(value) = object.borrow().fields.get(name).map(|val| (*val).clone()) {
            value
        } else if let Some(super_ptr) = &object.borrow().super_ptr {
            Object::get_field(super_ptr, name)
        } else {
            Value::Null
        }
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
