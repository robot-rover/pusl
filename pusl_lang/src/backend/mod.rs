use std::{cell::RefCell, collections::HashMap};

use garbage::{GcPointer, ManagedPool, MarkTrace};
use typemap::TypeMap;

use crate::backend::linearize::ByteCodeFile;
use crate::backend::object::{Object, ObjectPtr, Value};
use crate::parser::expression::Compare;
use std::cmp::Ordering;
use std::path::PathBuf;

use std::{
    fmt::{self, Debug},
    sync::mpsc::{Receiver, Sender},
};

pub mod argparse;
pub mod builtins;
pub mod debug;
pub mod linearize;
pub mod list;
pub mod object;

use debug::{DebugCommand, DebugResponse};
use fmt::Formatter;
use linearize::{OpCode, ResolvedFunction};

use self::object::{FunctionTarget, NativeFn};

pub struct BoundFunction {
    pub bound_values: Vec<Value>,
    pub target: &'static ResolvedFunction,
}

impl MarkTrace for BoundFunction {
    fn mark_children(&self) {
        self.bound_values
            .iter()
            .for_each(|value| value.mark_children())
    }
}

impl Debug for BoundFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            write!(f, "Bound{:?}", self.target)
        } else {
            self.target.fmt(f)?;
            write!(f, "\nBindings:")?;
            for (name, binding) in self
                .target
                .function
                .binds
                .iter()
                .zip(self.bound_values.iter())
            {
                write!(f, "\n    {}: {:?}", name, binding)?;
            }
            Ok(())
        }
    }
}

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

pub struct StackFrame {
    this_obj: Option<GcPointer<RefCell<Object>>>,
    bfunc: GcPointer<BoundFunction>,
    variables: Vec<VariableStack>,
    op_stack: Vec<Value>,
    index: usize,
}

impl StackFrame {
    fn from_function(bfunc: GcPointer<BoundFunction>, this_obj: Option<ObjectPtr>) -> Self {
        StackFrame {
            this_obj,
            bfunc,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        }
    }

    fn from_file(bfunc: GcPointer<BoundFunction>, gc: &RefCell<ManagedPool>) -> (Self, ObjectPtr) {
        let new_object = gc.borrow_mut().place_in_heap(Object::new());
        let frame = StackFrame {
            this_obj: Some(new_object.clone()),
            bfunc,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        };
        (frame, new_object)
    }

    pub fn get_code(&mut self) -> Option<OpCode> {
        let code = self.bfunc.target.function.get_code(self.index);
        self.index += 1;
        code
    }

    pub fn get_val(&mut self) -> usize {
        let value = self.bfunc.target.function.get_val(self.index);
        self.index += 1;
        value
    }

    pub fn get_cmp(&mut self) -> Compare {
        let value = self.bfunc.target.function.get_cmp(self.index);
        self.index += 1;
        value
    }

    pub fn get_assign_type(&mut self) -> bool {
        let value = self.bfunc.target.function.get_assign_type(self.index);
        self.index += 1;
        value
    }
}

pub struct ExecContext {
    pub resolve: fn(PathBuf) -> Option<ByteCodeFile>,
}

impl Default for ExecContext {
    fn default() -> Self {
        ExecContext { resolve: |_| None }
    }
}

fn process_bcf(
    bcf: ByteCodeFile,
    resolved_imports: &Vec<(PathBuf, ObjectPtr)>,
    gc: &RefCell<ManagedPool>,
) -> (StackFrame, (PathBuf, ObjectPtr)) {
    let ByteCodeFile {
        file,
        base_func,
        imports,
    } = bcf;
    let rfunc = base_func.resolve(resolved_imports, imports, gc);
    let bfunc = rfunc.bind(Vec::new(), gc);
    let (current_frame, import_object) = StackFrame::from_file(bfunc, gc);
    (current_frame, (file, import_object))
}

type DebugTuple = (Receiver<DebugCommand>, Sender<DebugResponse>);

pub struct ExecutionState<'a> {
    imports: Vec<(PathBuf, ObjectPtr)>,
    execution_stack: Vec<StackFrame>,
    current_frame: StackFrame,
    resolve_stack: Vec<ByteCodeFile>,
    gc: RefCell<ManagedPool>,
    builtins: HashMap<&'static str, Value>,
    builtin_data: TypeMap,
    registry: Vec<NativeFn<'a>>,
}

impl<'a> Debug for ExecutionState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let current_op = self
            .current_frame
            .bfunc
            .target
            .function
            .get_code(self.current_frame.index - 1)
            .unwrap();
        linearize::write_bytecode_line(
            (
                self.current_frame.index,
                &linearize::ByteCode::op(current_op),
            ),
            f,
            &mut (&self.current_frame.bfunc.target.function.code[self.current_frame.index..])
                .iter()
                .enumerate(),
            &self.current_frame.bfunc.target.function,
        )
    }
}

pub fn startup(main: ByteCodeFile, ctx: ExecContext) {
    let mut registry = Vec::new();
    let (builtins, builtin_data) = builtins::get_builtins(&mut registry);

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

    //TODO: Can we remove this refcell now?
    let gc = RefCell::new(ManagedPool::new());

    let top = resolve_stack.pop().unwrap();
    let (current_frame, resolution) = process_bcf(top, &resolved_imports, &gc);
    resolved_imports.push(resolution);

    let state = ExecutionState {
        imports: resolved_imports,
        execution_stack: Vec::new(),
        current_frame,
        resolve_stack,
        gc,
        builtins,
        builtin_data,
        registry,
    };

    let rstate = RefCell::new(state);

    execute(&rstate);
}

fn execute<'a>(st: &'a RefCell<ExecutionState<'a>>) -> Value {
    let mut native_fn_call: Option<(NativeFn, Vec<Value>, Option<Value>)> = None;
    loop {
        {
            let mut state = st.borrow_mut();
            let current_op = if let Some(op) = state.current_frame.get_code() {
                op
            } else {
                if let Some(mut parent_frame) = state.execution_stack.pop() {
                    parent_frame.op_stack.push(Value::Null);
                    state.current_frame = parent_frame;
                    continue;
                } else if let Some(parent_frame) = state.resolve_stack.pop() {
                    let (frame, resolution) = process_bcf(parent_frame, &state.imports, &state.gc);
                    state.current_frame = frame;
                    state.imports.push(resolution);
                    continue;
                } else {
                    return Value::Null;
                }
            };
            // TODO:
            // if true {
            //     println!("{:?}", state);
            // }
            match current_op {
                OpCode::Modulus => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(modulus(lhs, rhs));
                }
                OpCode::Literal => {
                    let pool_index = state.current_frame.get_val();
                    let literal = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_literal(pool_index);
                    let value = literal.into_value(&state.gc);
                    state.current_frame.op_stack.push(value);
                }
                OpCode::PushThis => {
                    let this_ref = state
                        .current_frame
                        .this_obj
                        .clone()
                        .expect("Cannot reference this");
                    state.current_frame.op_stack.push(Value::Object(this_ref));
                }
                OpCode::PushSelf => {
                    let self_ref = state.current_frame.bfunc.clone();
                    let this_ref = state.current_frame.this_obj.clone();
                    state
                        .current_frame
                        .op_stack
                        .push(Value::Function((FunctionTarget::Pusl(self_ref), this_ref)));
                }
                OpCode::PushReference => {
                    let pool_index = state.current_frame.get_val();
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
                    let value = state
                        .current_frame
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
                            state
                                .current_frame
                                .bfunc
                                .target
                                .function
                                .binds
                                .iter()
                                .position(|name| name == &reference_name)
                                .map(|index| state.current_frame.bfunc.bound_values[index].clone())
                        })
                        .or_else(|| {
                            state
                                .current_frame
                                .bfunc
                                .target
                                .imports
                                .iter()
                                .find(|&(name, _)| name.as_str() == reference_name)
                                .map(|(_, obj)| Value::Object(obj.clone()))
                        })
                        .or_else(|| state.builtins.get(reference_name.as_str()).cloned())
                        .expect(format!("Undeclared Variable \"{}\"", reference_name).as_str());
                    state.current_frame.op_stack.push(value);
                }
                OpCode::PushFunction => {
                    let pool_index = state.current_frame.get_val();
                    let rfunc = state.current_frame.bfunc.target.get_function(pool_index);
                    let bound_values = rfunc
                        .function
                        .binds
                        .iter()
                        .map(|name| {
                            state
                                .current_frame
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
                                .find(|var| &var.name == name)
                                .map(|var| var.value.clone())
                                .expect(format!("Undeclared Variable \"{}\"", name).as_str())
                        })
                        .collect();

                    let bfunc = rfunc.bind(bound_values, &state.gc);

                    state.current_frame.op_stack.push(Value::pusl_fn(bfunc));
                }
                OpCode::FunctionCall => {
                    let num_args = state.current_frame.get_val();
                    assert!(state.current_frame.op_stack.len() >= num_args);
                    let split_off_index = state.current_frame.op_stack.len() - num_args;
                    let args = state.current_frame.op_stack.split_off(split_off_index);
                    let function = state.current_frame.op_stack.pop().unwrap();
                    match function {
                        Value::Function((FunctionTarget::Pusl(reference), this)) => {
                            assert_eq!(reference.target.function.args.len(), args.len());
                            let arg_value_iter = args.into_iter();
                            let mut new_frame = StackFrame::from_function(reference, this);
                            for (name, value) in new_frame
                                .bfunc
                                .target
                                .function
                                .args
                                .iter()
                                .cloned()
                                .zip(arg_value_iter)
                            {
                                new_frame
                                    .variables
                                    .push(VariableStack::Variable(Variable { value, name }));
                            }
                            let old_frame = std::mem::replace(&mut state.current_frame, new_frame);
                            state.execution_stack.push(old_frame);
                        }
                        Value::Function((FunctionTarget::Native(handle), this)) => {
                            let this = this.map(|obj| Value::Object(obj));
                            let ptr = state
                                .registry
                                .get(handle)
                                .expect("Out of bounds function handle")
                                .clone();
                            native_fn_call = Some((ptr, args, this));
                            //let result = ptr(args, this, &mut state);
                        }
                        _ => panic!("Value must be a function to call"),
                    };
                }
                OpCode::FieldAccess => {
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let name_index = state.current_frame.get_val();
                    let name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(name_index);
                    let value = match value {
                        Value::Object(object) => {
                            let value = Object::get_field(&object, name.as_str());
                            match value {
                                Value::Function((target, None)) => {
                                    Value::Function((target, Some(object)))
                                }
                                other => other,
                            }
                        }
                        Value::String(_) => unimplemented!(),
                        other => panic!("Cannot access field of this value: {:?}", other),
                    };
                    state.current_frame.op_stack.push(value);
                }
                OpCode::Addition => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(addition(lhs, rhs));
                }
                OpCode::Subtraction => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(subtraction(lhs, rhs));
                }
                OpCode::Negate => {
                    let operand = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(negate(operand));
                }
                OpCode::Multiply => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(multiplication(lhs, rhs));
                }
                OpCode::Divide => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(division(lhs, rhs));
                }
                OpCode::DivideTruncate => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state
                        .current_frame
                        .op_stack
                        .push(truncate_division(lhs, rhs));
                }
                OpCode::Exponent => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(exponent(lhs, rhs));
                }
                OpCode::Compare => {
                    let op = state.current_frame.get_cmp();
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(compare(lhs, rhs, op));
                }
                OpCode::And => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(logic(lhs, rhs, true));
                }
                OpCode::Or => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(logic(lhs, rhs, false));
                }
                OpCode::ScopeUp => {
                    state
                        .current_frame
                        .variables
                        .push(VariableStack::ScopeBoundary);
                }
                OpCode::ScopeDown => {
                    while let Some(VariableStack::Variable(_)) = state.current_frame.variables.pop()
                    {
                    }
                }
                OpCode::Return => {
                    let return_value = state.current_frame.op_stack.pop().unwrap();
                    if let Some(mut parent_frame) = state.execution_stack.pop() {
                        parent_frame.op_stack.push(return_value);
                        state.current_frame = parent_frame;
                        continue;
                    } else if let Some(parent_frame) = state.resolve_stack.pop() {
                        let (frame, resolution) =
                            process_bcf(parent_frame, &state.imports, &state.gc);
                        state.current_frame = frame;
                        state.imports.push(resolution);
                        continue;
                    } else {
                        return return_value;
                    }
                }
                OpCode::ConditionalJump => {
                    let jump_index = state.current_frame.get_val();
                    let condition =
                        if let Value::Boolean(val) = state.current_frame.op_stack.pop().unwrap() {
                            val
                        } else {
                            panic!("ConditionalJump expects boolean");
                        };
                    if condition {
                        state.current_frame.index = jump_index;
                    }
                }
                OpCode::ComparisonJump => {
                    let greater_index = state.current_frame.get_val();
                    let less_index = state.current_frame.get_val();
                    let equal_index = state.current_frame.get_val();
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    let ordering = compare_numerical(lhs, rhs);
                    let index = match ordering {
                        Ordering::Less => less_index,
                        Ordering::Equal => equal_index,
                        Ordering::Greater => greater_index,
                    };
                    state.current_frame.index = index;
                }
                OpCode::Jump => {
                    let jump_index = state.current_frame.get_val();
                    state.current_frame.index = jump_index;
                }
                OpCode::Pop => {
                    state.current_frame.op_stack.pop().unwrap();
                }
                OpCode::IsNull => {
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let is_null = if let Value::Null = value { true } else { false };
                    state.current_frame.op_stack.push(Value::Boolean(is_null));
                }
                OpCode::Duplicate => {
                    let value = (*state.current_frame.op_stack.last().unwrap()).clone();
                    state.current_frame.op_stack.push(value);
                }
                OpCode::AssignReference => {
                    let pool_index = state.current_frame.get_val();
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
                    let is_let = state.current_frame.get_assign_type();
                    let value = state.current_frame.op_stack.pop().unwrap();
                    if is_let {
                        state
                            .current_frame
                            .variables
                            .push(VariableStack::Variable(Variable {
                                value,
                                name: reference_name,
                            }))
                    } else {
                        let variable = state
                            .current_frame
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
                    let pool_index = state.current_frame.get_val();
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
                    let is_let = state.current_frame.get_assign_type();
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let object = match state.current_frame.op_stack.pop().unwrap() {
                        Value::Object(ptr) => ptr,
                        other => panic!("Cannot Assign to field of {:?}", other),
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
                    let n = state.current_frame.get_val();
                    let len = state.current_frame.op_stack.len();
                    assert!(n <= len);
                    let mut range = state.current_frame.op_stack[(len - n)..len]
                        .iter()
                        .map(|val| val.clone())
                        .collect();
                    state.current_frame.op_stack.append(&mut range);
                }
                OpCode::PushBuiltin => {
                    let pool_index = state.current_frame.get_val();
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
                    let builtin = state
                        .builtins
                        .get(reference_name.as_str())
                        .expect("Missing Builtin")
                        .clone();
                    state.current_frame.op_stack.push(builtin);
                }
                OpCode::DuplicateDeep => {
                    let dup_index = state.current_frame.get_val();
                    let stack_index = state.current_frame.op_stack.len() - 1 - dup_index;
                    let value = state
                        .current_frame
                        .op_stack
                        .get(stack_index)
                        .expect("Invalid DuplicateDeep Index")
                        .clone();
                    state.current_frame.op_stack.push(value);
                }
                OpCode::Yield => {
                    assert!(state.current_frame.bfunc.target.function.is_generator);
                }
            }
        }
        if let Some((ptr, args, this)) = native_fn_call.take() {
            let result = ptr(args, this, st);
            st.borrow_mut().current_frame.op_stack.push(result);
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
            Value::Function((lhs_target, lhs_self)) => {
                if let Value::Function((rhs_target, rhs_self)) = rhs {
                    if lhs_self == rhs_self {
                        match lhs_target {
                            object::FunctionTarget::Native(lfun) => match rhs_target {
                                object::FunctionTarget::Native(rfun) => lfun == rfun,
                                object::FunctionTarget::Pusl(_) => false,
                            },
                            object::FunctionTarget::Pusl(lfun) => match rhs_target {
                                object::FunctionTarget::Native(_) => false,
                                object::FunctionTarget::Pusl(rfun) => lfun == rfun,
                            },
                        }
                    } else {
                        false
                    }
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
