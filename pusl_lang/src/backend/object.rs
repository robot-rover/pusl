use super::{BoundFunction, ExecStateRef, StackFrame};
use bitflags::_core::cell::RefCell;
use bitflags::_core::fmt::Formatter;
use fmt::Display;
use std::any::Any;

use garbage::{Gc, MarkTrace};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Debug;

pub type ObjectPtr = Gc<RefCell<dyn Object>>;
pub type StringPtr = Gc<String>;
pub type NativeFn<'a> = fn(Vec<Value>, Option<Value>, ExecStateRef<'a>) -> Value;
pub type FnPtr = Gc<BoundFunction>;
pub type GeneratorFn = Gc<StackFrame>;
pub type MethodPtr = (FunctionTarget, Option<ObjectPtr>);

pub type NativeFnHandle = usize;

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionTarget {
    Native(NativeFnHandle),
    Pusl(FnPtr),
}

#[derive(Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(StringPtr),
    Function(MethodPtr),
    Object(ObjectPtr),
}

struct ObjectFmtWrapper<'a>(&'a ObjectPtr);

impl<'a> Debug for ObjectFmtWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.try_borrow() {
            Ok(borrow) => Debug::fmt(&*borrow, f),
            Err(_) => f
                .debug_struct("Object")
                .field("cannot_borrow", &true)
                .finish_non_exhaustive(),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.debug_tuple("Null").finish(),
            Value::Boolean(val) => f.debug_tuple("Boolean").field(val).finish(),
            Value::Integer(val) => f.debug_tuple("Integer").field(val).finish(),
            Value::Float(val) => f.debug_tuple("Float").field(val).finish(),
            Value::String(val) => f.debug_tuple("String").field(&**val).finish(),
            Value::Function((function, this)) => {
                let mut debug = f.debug_struct("Function");
                match function {
                    FunctionTarget::Native(native) => debug.field("native", native),
                    FunctionTarget::Pusl(pusl) => debug.field("pusl", &**pusl),
                };
                if let Some(this) = this {
                    debug.field("this", &ObjectFmtWrapper(this));
                }
                debug.finish()
            }
            Value::Object(obj) => f
                .debug_tuple("Object")
                .field(&ObjectFmtWrapper(obj))
                .finish(),
        }
    }
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
            Value::Function((FunctionTarget::Pusl(_), _)) => "Pusl_Function",
            Value::Function((FunctionTarget::Native(_), _)) => "Native_Function",
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
    fn mark_trace(&self) {
        if let Value::Object(object) = self {
            object.mark_trace()
        }
    }
}

impl MarkTrace for PuslObject {
    fn mark_trace(&self) {
        if let Some(super_ptr) = &self.super_ptr {
            super_ptr.mark_trace();
        }
        self.fields.iter().for_each(|(_, v)| {
            if let Value::Object(ptr) = v {
                ptr.mark_trace();
            }
        })
    }
}

pub fn is_instance_of(obj: ObjectPtr, parent: &ObjectPtr) -> bool {
    let mut current = obj;
    loop {
        if &current == parent {
            return true;
        }
        let pusl_obj = current
            .borrow()
            .get_native_data()
            .downcast_ref::<PuslObject>()
            .and_then(|pusl_obj| pusl_obj.super_ptr.clone());
        if let Some(super_ptr) = pusl_obj {
            current = super_ptr
        } else {
            return false;
        }
    }
}

//Todo: The debug impl really should be custom
pub struct PuslObject {
    super_ptr: Option<ObjectPtr>,
    fields: HashMap<String, Value>,
}

impl fmt::Debug for PuslObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("super_ptr", &self.super_ptr)
            .field("fields", &self.fields)
            .finish()
    }
}

pub trait Object: MarkTrace + Debug {
    fn assign_field(&mut self, name: &str, value: Value, is_let: bool);
    fn get_field(&self, name: &str) -> Value;
    fn get_native_data(&self) -> &dyn Any;
    fn get_native_data_mut(&mut self) -> &mut dyn Any;
}

macro_rules! impl_native_data {
    () => {
        fn get_native_data(&self) -> &dyn Any {
            self
        }
        fn get_native_data_mut(&mut self) -> &mut dyn Any {
            self
        }
    };
}

impl PuslObject {
    pub fn new() -> RefCell<Self> {
        let object = PuslObject {
            super_ptr: None,
            fields: HashMap::new(),
        };
        RefCell::new(object)
    }

    pub fn new_with_parent(parent: ObjectPtr) -> RefCell<Self> {
        let object = PuslObject {
            super_ptr: Some(parent),
            fields: HashMap::new(),
        };
        RefCell::new(object)
    }
}

impl Object for PuslObject {
    fn assign_field(&mut self, name: &str, value: Value, is_let: bool) {
        if name == "super" {
            match value {
                Value::Object(object_ptr) => self.super_ptr = Some(object_ptr),
                Value::Null => panic!("Cannot Remove Super Object"),
                _ => panic!("Super Object must be an Object"),
            }
        } else {
            if is_let {
                self.fields.insert(name.to_string(), value);
            } else {
                let entry = self.fields.get_mut(name);
                if let Some(old_value) = entry {
                    *old_value = value;
                } else {
                    panic!("Cannot assign to non-existent field without let")
                }
            }
        }
    }

    fn get_field(&self, name: &str) -> Value {
        if name == "super" {
            match &self.super_ptr {
                Some(super_obj) => Value::Object(super_obj.clone()),
                None => Value::Null,
            }
        } else {
            //TODO: Bad Recursion
            if let Some(value) = self.fields.get(name).map(|val| (*val).clone()) {
                value
            } else if let Some(super_ptr) = &self.super_ptr {
                super_ptr.borrow().get_field(name)
            } else {
                Value::Null
            }
        }
    }

    impl_native_data!();
}
