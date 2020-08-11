use std::cell::RefCell;
use std::thread::LocalKey;

use garbage::{GcPointer, ManagedPool};

use crate::backend::linearize::{resolve, ByteCodeFile, Function, OpCode};
use crate::backend::object::{Object, ObjectPtr, Value};
use crate::parser::expression::Compare;
use std::cmp::Ordering;
use std::io;
use std::path::PathBuf;

use log::trace;
use std::sync::mpsc::{Receiver, Sender};

pub mod argparse;
pub mod builtins;
pub mod debug;
pub mod linearize;
pub mod list;
pub mod object;

use debug::{DebugCommand, DebugResponse};

pub type RFunction = Function<&'static Vec<(String, ObjectPtr)>>;

#[derive(Debug)]
enum VariableStack {
    Variable(Variable),
    ScopeBoundary,
}

#[derive(Debug)]
struct Variable {
    value: Value,
    name: String,
}

struct StackFrame {
    this_obj: Option<GcPointer<RefCell<Object>>>,
    function: &'static RFunction,
    variables: Vec<VariableStack>,
    op_stack: Vec<Value>,
    index: usize,
}

impl StackFrame {
    fn from_function(function: &'static RFunction) -> Self {
        StackFrame {
            this_obj: None,
            function,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        }
    }

    fn from_method(function: &'static RFunction, this_obj: ObjectPtr) -> Self {
        StackFrame {
            this_obj: Some(this_obj),
            function,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        }
    }

    fn from_file(function: &'static RFunction) -> (Self, ObjectPtr) {
        let new_object = GC.with(|gc| gc.borrow_mut().place_in_heap(Object::new()));
        let frame = StackFrame {
            this_obj: Some(new_object.clone()),
            function,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        };
        (frame, new_object)
    }

    pub fn get_code(&mut self) -> Option<OpCode> {
        let code = self.function.get_code(self.index);
        self.index += 1;
        code
    }

    pub fn get_val(&mut self) -> usize {
        let value = self.function.get_val(self.index);
        self.index += 1;
        value
    }

    pub fn get_cmp(&mut self) -> Compare {
        let value = self.function.get_cmp(self.index);
        self.index += 1;
        value
    }

    pub fn get_assign_type(&mut self) -> bool {
        let value = self.function.get_assign_type(self.index);
        self.index += 1;
        value
    }
}

thread_local! {
    pub static GC: RefCell<ManagedPool> = RefCell::new(ManagedPool::new());
    pub static STDOUT: RefCell<Box<dyn io::Write>> = RefCell::new(Box::new(io::stdout()));
}

pub type GcPoolRef = &'static LocalKey<RefCell<ManagedPool>>;

pub struct ExecContext {
    pub resolve: fn(PathBuf) -> Option<ByteCodeFile>,
}

impl Default for ExecContext {
    fn default() -> Self {
        ExecContext { resolve: |_| None }
    }
}

fn process_bcf(bcf: ByteCodeFile, resolved_imports: &mut Vec<(PathBuf, ObjectPtr)>) -> StackFrame {
    let ByteCodeFile {
        file,
        base_func,
        imports,
    } = bcf;
    let base_func = resolve(base_func, resolved_imports as &_, imports, &GC);
    let (current_frame, import_object) = StackFrame::from_file(base_func);
    resolved_imports.push((file, import_object));
    current_frame
}

type DebugTuple = (Receiver<DebugCommand>, Sender<DebugResponse>);

pub fn execute(main: ByteCodeFile, ctx: ExecContext, mut debug: Option<DebugTuple>) {
    let mut stop_index = None;
    if let Some(tuple) = &mut debug {
        tuple.1.send(DebugResponse::Paused(0)).unwrap();
        match tuple.0.recv().unwrap() {
            DebugCommand::RunToIndex(index) => stop_index = Some(index),
            DebugCommand::Run => {}
        }
    }
    if let Some(index) = stop_index {
        println!("running to index: {}", index);
    }

    let builtins = builtins::get_builtins();

    let ExecContext { resolve } = ctx;
    let mut resolved_imports = Vec::<(PathBuf, ObjectPtr)>::new();
    let mut resolve_stack = vec![main];
    let mut index = 0;
    // TODO: Don't clone here
    while index < resolve_stack.len() {
        let mut append = Vec::new();
        for import in &resolve_stack[index].imports {
            if !resolve_stack.iter().any(|bcf| bcf.file == import.path) {
                let new_bcf = resolve(import.path.clone())
                    .expect(format!("Unable to resolve import {}", import.path.display()).as_str());
                append.push(new_bcf);
            }
        }
        resolve_stack.append(&mut append);
        index += 1;
    }

    let mut ex_stack = Vec::<StackFrame>::new();
    let top = resolve_stack.pop().unwrap();
    let mut current_frame = process_bcf(top, &mut resolved_imports);

    loop {
        if let Some(index) = stop_index {
            if current_frame.index >= index {
                if let Some(tuple) = &mut debug {
                    let operation = current_frame.function.code.get(current_frame.index);
                    if let Some(code) = operation {
                        println!("{}: {:?}", current_frame.index, code.as_op());
                    }
                    println!("Variables: ");
                    for variable in &current_frame.variables {
                        match variable {
                            VariableStack::Variable(Variable { name, value }) => {
                                println!("\t{}: {:?}", name, value)
                            }
                            VariableStack::ScopeBoundary => println!("\t----"),
                        }
                    }
                    println!("Stack: ");
                    for value in &current_frame.op_stack {
                        println!("\t{:?}", value);
                    }
                    tuple
                        .1
                        .send(DebugResponse::Paused(current_frame.index))
                        .unwrap();
                    match tuple.0.recv().unwrap() {
                        DebugCommand::RunToIndex(index) => stop_index = Some(index),
                        DebugCommand::Run => stop_index = None,
                    }
                }
            }
        }
        let current_op = if let Some(op) = current_frame.get_code() {
            if debug.is_some() {
                trace!("{}: {:?}", current_frame.index - 1, op);
            }
            op
        } else {
            if let Some(mut parent_frame) = ex_stack.pop() {
                parent_frame.op_stack.push(Value::Null);
                current_frame = parent_frame;
                continue;
            } else if let Some(parent_frame) = resolve_stack.pop() {
                current_frame = process_bcf(parent_frame, &mut resolved_imports);
                continue;
            } else {
                if let Some(tuple) = &mut debug {
                    tuple.1.send(DebugResponse::Done).unwrap()
                }
                return;
            }
        };
        // TODO:
        // if DEBUG {
        //     write_bytecode_line((current_frame.index, &ByteCode::op(current_op)), &mut stdout_handle, &mut current_frame.function.code[current_frame.index..], current_frame.function)
        // }
        match current_op {
            OpCode::Modulus => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(modulus(lhs, rhs));
            }
            OpCode::Literal => {
                let pool_index = current_frame.get_val();
                current_frame.op_stack.push(
                    current_frame
                        .function
                        .get_literal(pool_index)
                        .into_value(&GC),
                )
            }
            OpCode::PushSelf => {
                let self_ref = current_frame
                    .this_obj
                    .clone()
                    .expect("Cannot reference self");
                current_frame.op_stack.push(Value::Object(self_ref));
            }
            OpCode::PushReference => {
                let pool_index = current_frame.get_val();
                let reference_name = current_frame.function.get_reference(pool_index);
                let value = current_frame
                    .variables
                    .iter_mut()
                    .rev()
                    .filter_map(|var_stack| {
                        if let VariableStack::Variable(var) = var_stack {
                            Some(var)
                        } else {
                            None
                        }
                    })
                    .find(|var| var.name == reference_name)
                    .map(|var| var.value.clone())
                    .or_else(|| {
                        current_frame
                            .function
                            .resolved
                            .iter()
                            .find(|&(name, _)| name.as_str() == reference_name)
                            .map(|(_, obj)| Value::Object(obj.clone()))
                    })
                    .or_else(|| builtins.get(reference_name.as_str()).cloned())
                    .expect(format!("Undeclared Variable \"{}\"", reference_name).as_str());
                current_frame.op_stack.push(value);
            }
            OpCode::PushFunction => {
                let pool_index = current_frame.get_val();
                current_frame.op_stack.push(Value::Function(
                    current_frame.function.get_function(pool_index),
                ));
            }
            OpCode::FunctionCall => {
                let num_args = current_frame.get_val();
                let mut args = Vec::with_capacity(num_args);
                for _ in 0..num_args {
                    args.push(current_frame.op_stack.pop().unwrap());
                }
                let function = current_frame.op_stack.pop().unwrap();
                match function {
                    Value::Function(reference) => {
                        assert_eq!(reference.args.len(), args.len());
                        let mut new_frame = StackFrame::from_function(reference);
                        for name in reference.args.iter().cloned() {
                            let value = args.pop().expect("Wrong Number of arguments for function");
                            new_frame
                                .variables
                                .push(VariableStack::Variable(Variable { value, name }));
                        }
                        assert!(args.is_empty(), "Wrong number of arguments for function");
                        let old_frame = std::mem::replace(&mut current_frame, new_frame);
                        ex_stack.push(old_frame);
                    }
                    Value::Native(ptr) => {
                        let result = ptr(args, None, &GC);
                        current_frame.op_stack.push(result);
                    }
                    _ => panic!("Value must be a function to call"),
                };
            }
            OpCode::MethodCall => {
                let num_args = current_frame.get_val();
                let mut args = Vec::with_capacity(num_args);
                for _ in 0..num_args {
                    args.push(current_frame.op_stack.pop().unwrap());
                }
                let function = current_frame.op_stack.pop().unwrap();
                let value = current_frame.op_stack.pop().unwrap();
                match function {
                    Value::Function(reference) => {
                        assert_eq!(reference.args.len(), args.len());
                        let this_obj = if let Value::Object(ptr) = value {
                            ptr
                        } else {
                            panic!("Cannot call method on Non Object")
                        };
                        let mut new_frame = StackFrame::from_method(reference, this_obj);
                        for name in reference.args.iter().cloned() {
                            let value = args.pop().expect("Wrong Number of arguments for function");
                            new_frame
                                .variables
                                .push(VariableStack::Variable(Variable { value, name }));
                        }
                        assert!(args.is_empty(), "Wrong number of arguments for function");
                        let old_frame = std::mem::replace(&mut current_frame, new_frame);
                        ex_stack.push(old_frame);
                    }
                    Value::Native(ptr) => {
                        let result = ptr(args, Some(value), &GC);
                        current_frame.op_stack.push(result);
                    }
                    _ => panic!("Value must be a function to call"),
                };
            }
            OpCode::FieldAccess => {
                let value = current_frame.op_stack.pop().unwrap();
                let name_index = current_frame.get_val();
                let name = current_frame.function.get_reference(name_index);
                let value = match value {
                    Value::Object(object) => Object::get_field(object, name.as_str()),
                    Value::String(_) => unimplemented!(),
                    _ => panic!("Cannot access field of this value"),
                };
                current_frame.op_stack.push(value);
            }
            OpCode::Addition => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(addition(lhs, rhs));
            }
            OpCode::Subtraction => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(subtraction(lhs, rhs));
            }
            OpCode::Negate => {
                let operand = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(negate(operand));
            }
            OpCode::Multiply => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(multiplication(lhs, rhs));
            }
            OpCode::Divide => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(division(lhs, rhs));
            }
            OpCode::DivideTruncate => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(truncate_division(lhs, rhs));
            }
            OpCode::Exponent => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(exponent(lhs, rhs));
            }
            OpCode::Compare => {
                let op = current_frame.get_cmp();
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(compare(lhs, rhs, op));
            }
            OpCode::And => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(logic(lhs, rhs, true));
            }
            OpCode::Or => {
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                current_frame.op_stack.push(logic(lhs, rhs, false));
            }
            OpCode::ScopeUp => {
                current_frame.variables.push(VariableStack::ScopeBoundary);
            }
            OpCode::ScopeDown => {
                while let Some(VariableStack::Variable(_)) = current_frame.variables.pop() {}
            }
            OpCode::Return => {
                let return_value = current_frame.op_stack.pop().unwrap();
                if let Some(mut parent_frame) = ex_stack.pop() {
                    parent_frame.op_stack.push(return_value);
                    current_frame = parent_frame;
                    continue;
                } else if let Some(parent_frame) = resolve_stack.pop() {
                    current_frame = process_bcf(parent_frame, &mut resolved_imports);
                    continue;
                } else {
                    return;
                }
            }
            OpCode::ConditionalJump => {
                let jump_index = current_frame.get_val();
                let condition = if let Value::Boolean(val) = current_frame.op_stack.pop().unwrap() {
                    val
                } else {
                    panic!("ConditionalJump expects boolean");
                };
                if condition {
                    current_frame.index = jump_index;
                }
            }
            OpCode::ComparisonJump => {
                let greater_index = current_frame.get_val();
                let less_index = current_frame.get_val();
                let equal_index = current_frame.get_val();
                let rhs = current_frame.op_stack.pop().unwrap();
                let lhs = current_frame.op_stack.pop().unwrap();
                let ordering = compare_numerical(lhs, rhs);
                let index = match ordering {
                    Ordering::Less => less_index,
                    Ordering::Equal => equal_index,
                    Ordering::Greater => greater_index,
                };
                current_frame.index = index;
            }
            OpCode::Jump => {
                let jump_index = current_frame.get_val();
                current_frame.index = jump_index;
            }
            OpCode::Pop => {
                current_frame.op_stack.pop().unwrap();
            }
            OpCode::IsNull => {
                let value = current_frame.op_stack.pop().unwrap();
                let is_null = if let Value::Null = value { true } else { false };
                current_frame.op_stack.push(Value::Boolean(is_null));
            }
            OpCode::Duplicate => {
                let value = (*current_frame.op_stack.last().unwrap()).clone();
                current_frame.op_stack.push(value);
            }
            OpCode::AssignReference => {
                let pool_index = current_frame.get_val();
                let reference_name = current_frame.function.get_reference(pool_index);
                let is_let = current_frame.get_assign_type();
                let value = current_frame.op_stack.pop().unwrap();
                if is_let {
                    current_frame
                        .variables
                        .push(VariableStack::Variable(Variable {
                            value,
                            name: reference_name,
                        }))
                } else {
                    let variable = current_frame
                        .variables
                        .iter_mut()
                        .rev()
                        .filter_map(|var_stack| {
                            if let VariableStack::Variable(var) = var_stack {
                                Some(var)
                            } else {
                                None
                            }
                        })
                        .find(|var| var.name == reference_name)
                        .expect("Non-Let Assignment on undeclared variable");
                    variable.value = value;
                }
            }
            OpCode::AssignField => {
                let pool_index = current_frame.get_val();
                let reference_name = current_frame.function.get_reference(pool_index);
                let is_let = current_frame.get_assign_type();
                let value = current_frame.op_stack.pop().unwrap();
                let object = if let Value::Object(ptr) = current_frame.op_stack.pop().unwrap() {
                    ptr
                } else {
                    panic!("Cannot Assign to non-object")
                };

                if is_let {
                    (*object).borrow_mut().let_field(reference_name, value);
                } else {
                    (*object)
                        .borrow_mut()
                        .assign_field(reference_name.as_str(), value);
                }
            }
            OpCode::DuplicateMany => {
                let n = current_frame.get_val();
                let len = current_frame.op_stack.len();
                assert!(n <= len);
                let mut range = current_frame.op_stack[(len - n)..len]
                    .iter()
                    .map(|val| val.clone())
                    .collect();
                current_frame.op_stack.append(&mut range);
            }
            OpCode::PushBuiltin => {
                let pool_index = current_frame.get_val();
                let reference_name = current_frame.function.get_reference(pool_index);
                let builtin = builtins
                    .get(reference_name.as_str())
                    .expect("Missing Builtin")
                    .clone();
                current_frame.op_stack.push(builtin);
            }
            OpCode::DuplicateDeep => {
                let dup_index = current_frame.get_val();
                let stack_index = current_frame.op_stack.len() - 1 - dup_index;
                let value = current_frame
                    .op_stack
                    .get(stack_index)
                    .expect("Invalid DuplicateDeep Index")
                    .clone();
                current_frame.op_stack.push(value);
            }
        }
    }
}

fn logic(lhs: Value, rhs: Value, is_and: bool) -> Value {
    match lhs {
        Value::Boolean(lhs) => {
            if let Value::Boolean(rhs) = rhs {
                let result = if is_and { lhs & rhs } else { lhs | rhs };
                Value::Boolean(result)
            } else {
                panic!("Use Logical Operator with Boolean or Integer")
            }
        }
        Value::Integer(lhs) => {
            if let Value::Integer(rhs) = rhs {
                let result = if is_and { lhs & rhs } else { lhs | rhs };
                Value::Integer(result)
            } else {
                panic!("Use Logical Operator with Boolean or Integer")
            }
        }
        _ => panic!("Use Logical Operator with Boolean or Integer"),
    }
}

#[inline]
fn modulus(lhs: Value, rhs: Value) -> Value {
    let lhs = if let Value::Integer(value) = lhs {
        value
    } else {
        panic!("Modulus only works with Integral operands")
    };

    let rhs = if let Value::Integer(value) = rhs {
        value
    } else {
        panic!("Modulus only works with Integral operands")
    };
    return Value::Integer(lhs % rhs);
}

#[inline]
fn addition(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Integer(lhs + rhs),
            Value::Float(rhs) => Value::Float(lhs as f64 + rhs),
            _ => panic!("Invalid Operand for Addition"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs + rhs as f64),
            Value::Float(rhs) => Value::Float(lhs + rhs),
            _ => panic!("Invalid Operand for Addition"),
        },
        _ => panic!("Invalid Operand for Addition"),
    }
}

#[inline]
fn subtraction(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Integer(lhs - rhs),
            Value::Float(rhs) => Value::Float(lhs as f64 - rhs),
            _ => panic!("Invalid Operand for Subtraction"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs - rhs as f64),
            Value::Float(rhs) => Value::Float(lhs - rhs),
            _ => panic!("Invalid Operand for Subtraction"),
        },
        _ => panic!("Invalid Operand for Subtraction"),
    }
}

#[inline]
fn multiplication(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Integer(lhs * rhs),
            Value::Float(rhs) => Value::Float(lhs as f64 * rhs),
            _ => panic!("Invalid Operand for Multiplication"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs * rhs as f64),
            Value::Float(rhs) => Value::Float(lhs * rhs),
            _ => panic!("Invalid Operand for Multiplication"),
        },
        _ => panic!("Invalid Operand for Multiplication"),
    }
}

#[inline]
fn division(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs as f64 / rhs as f64),
            Value::Float(rhs) => Value::Float(lhs as f64 / rhs),
            _ => panic!("Invalid Operand for Division"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs / rhs as f64),
            Value::Float(rhs) => Value::Float(lhs / rhs),
            _ => panic!("Invalid Operand for Division"),
        },
        _ => panic!("Invalid Operand for Division"),
    }
}

#[inline]
fn truncate_division(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Integer(lhs / rhs),
            Value::Float(rhs) => Value::Integer((lhs as f64 / rhs) as i64),
            _ => panic!("Invalid Operand for TruncDivision"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Integer((lhs / rhs as f64) as i64),
            Value::Float(rhs) => Value::Integer((lhs / rhs) as i64),
            _ => panic!("Invalid Operand for TruncDivision"),
        },
        _ => panic!("Invalid Operand for TruncDivision"),
    }
}

#[inline]
fn exponent(lhs: Value, rhs: Value) -> Value {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float((lhs as f64).powi(rhs as i32)),
            Value::Float(rhs) => Value::Float((lhs as f64).powf(rhs)),
            _ => panic!("Invalid Operand for Exponent"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => Value::Float(lhs.powi(rhs as i32)),
            Value::Float(rhs) => Value::Float(lhs.powf(rhs)),
            _ => panic!("Invalid Operand for Exponent"),
        },
        _ => panic!("Invalid Operand for Exponent"),
    }
}

#[inline]
fn negate(operand: Value) -> Value {
    match operand {
        Value::Boolean(val) => Value::Boolean(!val),
        Value::Integer(val) => Value::Integer(-val),
        Value::Float(val) => Value::Float(-val),
        _ => panic!("Invalid Operand for Negation"),
    }
}

#[inline]
fn compare(lhs: Value, rhs: Value, compare: Compare) -> Value {
    let equality = match compare {
        Compare::Equal => Some(false),
        Compare::NotEqual => Some(true),
        _ => None,
    };

    let result = if let Some(invert) = equality {
        let is_equal = match lhs {
            Value::Null => {
                if let Value::Null = rhs {
                    true
                } else {
                    false
                }
            }
            Value::Boolean(lhs) => {
                if let Value::Boolean(rhs) = rhs {
                    lhs == rhs
                } else {
                    false
                }
            }
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => lhs == rhs,
                Value::Float(rhs) => lhs as f64 == rhs,
                _ => false,
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => lhs == rhs as f64,
                Value::Float(rhs) => lhs == rhs,
                _ => false,
            },
            Value::String(lhs) => {
                if let Value::String(rhs) = rhs {
                    *lhs == *rhs
                } else {
                    false
                }
            }
            Value::Function(lhs) => {
                if let Value::Function(rhs) = rhs {
                    lhs as *const _ == rhs as *const _
                } else {
                    false
                }
            }
            Value::Object(lhs) => {
                if let Value::Object(rhs) = rhs {
                    lhs == rhs
                } else {
                    false
                }
            }
            Value::Native(lhs) => {
                if let Value::Native(rhs) = rhs {
                    lhs == rhs
                } else {
                    false
                }
            }
        };
        is_equal ^ invert
    } else {
        let cmp = compare_numerical(lhs, rhs);

        match compare {
            Compare::Less => match cmp {
                Ordering::Less => true,
                Ordering::Equal => false,
                Ordering::Greater => false,
            },
            Compare::LessEqual => match cmp {
                Ordering::Less => true,
                Ordering::Equal => true,
                Ordering::Greater => false,
            },
            Compare::Greater => match cmp {
                Ordering::Less => false,
                Ordering::Equal => false,
                Ordering::Greater => true,
            },
            Compare::GreaterEqual => match cmp {
                Ordering::Less => false,
                Ordering::Equal => true,
                Ordering::Greater => true,
            },
            _ => panic!("Invariant"),
        }
    };

    Value::Boolean(result)
}

fn compare_numerical(lhs: Value, rhs: Value) -> Ordering {
    match lhs {
        Value::Integer(lhs) => match rhs {
            Value::Integer(rhs) => lhs.cmp(&rhs),
            Value::Float(rhs) => (lhs as f64).partial_cmp(&rhs).expect("Comparison Failed!"),
            _ => panic!("Cannot Compare non-numeric types"),
        },
        Value::Float(lhs) => match rhs {
            Value::Integer(rhs) => lhs.partial_cmp(&(rhs as f64)).expect("Comparison Failed!"),
            Value::Float(rhs) => lhs.partial_cmp(&rhs).expect("Comparison Failed!"),
            _ => panic!("Cannot Compare non-numeric types"),
        },
        _ => panic!("Cannot Compare non-numeric types"),
    }
}
