use std::{cell::RefCell, collections::HashMap, env, mem};

use anymap::AnyMap;
use garbage::{Gc, ManagedPool, MarkTrace};

use crate::backend::linearize::{ByteCodeFile, ErrorCatch};
use crate::backend::object::{is_instance_of, FnPtr, Object, ObjectPtr, PuslObject, Value};
use crate::parser::expression::Compare;
use std::cmp::Ordering;
use std::path::PathBuf;

use std::fmt::{self, Debug};

#[macro_use]
pub mod object;
pub mod argparse;
pub mod builtins;
pub mod debug;
pub mod generator;
pub mod linearize;
pub mod list;

use fmt::Formatter;
use std::ops::Deref;

use crate::backend::ExecuteReturn::{Return, Yield};
use linearize::{OpCode, ResolvedFunction};

use self::object::{FunctionTarget, NativeFn};

// TODO: Convert Self references to use bound values idx 0
pub struct BoundFunction {
    pub bound_values: Vec<Value>,
    pub target: &'static ResolvedFunction,
}

impl MarkTrace for BoundFunction {
    fn mark_trace(&self) {
        self.bound_values
            .iter()
            .for_each(|value| value.mark_trace())
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

#[derive(Debug)]
pub struct StackFrame {
    this_obj: Option<ObjectPtr>,
    bfunc: FnPtr,
    variables: Vec<VariableStack>,
    op_stack: Vec<Value>,
    index: usize,
}

impl StackFrame {
    fn from_function(bfunc: FnPtr, this_obj: Option<ObjectPtr>) -> Self {
        StackFrame {
            this_obj,
            bfunc,
            variables: vec![],
            op_stack: vec![],
            index: 0,
        }
    }

    fn from_file(bfunc: FnPtr, gc: &mut ManagedPool) -> (Self, ObjectPtr) {
        let to_insert = PuslObject::new();
        let new_object = gc.place_in_heap(to_insert) as Gc<RefCell<dyn Object>>;

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
        code.as_ref().map(|(_, new_offset)| self.index = *new_offset);
        code.map(|(code, _)| code)
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
    path: PathBuf,
    resolved_imports: &Vec<(PathBuf, ObjectPtr)>,
    gc: &mut ManagedPool,
) -> (StackFrame, (PathBuf, ObjectPtr)) {
    let ByteCodeFile { base_func, imports } = bcf;
    let rfunc = base_func.resolve(resolved_imports, imports, gc);
    let bfunc = rfunc.bind(Vec::new(), gc);
    let (current_frame, import_object) = StackFrame::from_file(bfunc, gc);
    (current_frame, (path, import_object))
}

pub struct ExecutionState<'a> {
    imports: Vec<(PathBuf, ObjectPtr)>,
    execution_stack: Vec<StackFrame>,
    current_frame: StackFrame,
    resolve_stack: Vec<(PathBuf, ByteCodeFile)>,
    gc: ManagedPool,
    builtins: HashMap<&'static str, Value>,
    builtin_data: AnyMap,
    registry: Vec<NativeFn<'a>>,
}

impl<'a> Debug for ExecutionState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let current_op = self
            .current_frame
            .bfunc
            .target
            .function
            .code
            .get_offset(self.current_frame.index);
        if let Some(current_op) = current_op {
            current_op.0.format_opcode(self.current_frame.index, f, &self.current_frame.bfunc.target.function)
        } else {
            writeln!(f, "out of bounds")
        }
    }
}

pub fn startup(main: ByteCodeFile, main_path: PathBuf, ctx: ExecContext) {
    let mut registry = Vec::new();
    let (builtins, builtin_data) = builtins::get_builtins(&mut registry);

    let ExecContext { resolve } = ctx;
    let mut resolved_imports = Vec::<(PathBuf, ObjectPtr)>::new();
    let mut resolve_stack = vec![(main_path, main)];
    let mut index = 0;
    // TODO: Don't clone here
    while index < resolve_stack.len() {
        let mut append = Vec::new();
        for import in &resolve_stack[index].1.imports {
            if !resolve_stack
                .iter()
                .any(|(path, _bcf)| path == &import.path)
            {
                let new_bcf = resolve(import.path.clone()).unwrap_or_else(|| {
                    panic!("Unable to resolve import {}", import.path.display())
                });
                append.push((import.path.clone(), new_bcf));
            }
        }
        resolve_stack.append(&mut append);
        index += 1;
    }

    //TODO: Can we remove this refcell now?
    let mut gc = ManagedPool::new();

    let (main_path, top) = resolve_stack.pop().unwrap();
    let (current_frame, resolution) = process_bcf(top, main_path, &resolved_imports, &mut gc);
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

    let result = execute(&rstate);
    match result {
        Return(ret_val) => println!("Execution Returned {:?}", ret_val),
        Yield(yield_val) => println!("Execution Yielded {:?}", yield_val),
        ExecuteReturn::Error(error) => println!("Uncaught Error {:?}", error),
    }
}

enum ExecuteReturn {
    Return(Value),
    Yield(Value),
    Error(Value),
}

fn execute<'a: 'b, 'b>(st: &'a RefCell<ExecutionState<'b>>) -> ExecuteReturn {
    let mut current_catch: Option<(ObjectPtr, ErrorCatch)> = None;
    loop {
        let mut native_fn_call: Option<(NativeFn, Vec<Value>, Option<Value>)> = None;
        {
            let mut state = st.borrow_mut();
            let current_idx = state.current_frame.index;
            if let Some((error, catch)) = current_catch.take() {
                if current_idx == catch.yoink {
                    let filter = state.current_frame.op_stack.pop().unwrap();
                    if let Value::Object(object_ptr) = filter {
                        if is_instance_of(error.clone(), &object_ptr) {
                            state
                                .current_frame
                                .variables
                                .push(VariableStack::Variable(Variable {
                                    value: Value::Object(error),
                                    name: catch.variable_name,
                                }))
                        } else {
                            match unwind_stack(&mut state, current_idx, error) {
                                Ok(error_catch) => current_catch = Some(error_catch),
                                Err(ret_val) => return ExecuteReturn::Error(ret_val),
                            }
                            continue;
                        }
                    }
                } else {
                    current_catch = Some((error, catch));
                }
            }

            if env::var("PUSL_TRACE").is_ok() {
                println!("{:?}", state);
            }
            if env::var("PUSL_TRACE_VAR").is_ok() {
                println!("{:?}", &state.current_frame.op_stack);
            }

            let current_op = if let Some(op) = state.current_frame.get_code() {
                op
            } else {
                if let Some(mut parent_frame) = state.execution_stack.pop() {
                    parent_frame.op_stack.push(Value::Null);
                    state.current_frame = parent_frame;
                    continue;
                } else if let Some((path, parent_frame)) = state.resolve_stack.pop() {
                    let (frame, resolution) = {
                        let ExecutionState { imports, gc, .. } = &mut *state;
                        process_bcf(parent_frame, path, imports, gc)
                    };
                    state.current_frame = frame;
                    state.imports.push(resolution);
                    continue;
                } else {
                    return Return(Value::Null);
                }
            };

            match current_op {
                OpCode::Modulus => {
                    let rhs = state.current_frame.op_stack.pop().unwrap();
                    let lhs = state.current_frame.op_stack.pop().unwrap();
                    state.current_frame.op_stack.push(modulus(lhs, rhs));
                }
                OpCode::Literal(pool_index) => {
                    let literal = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_literal(pool_index);
                    let value = literal.into_value(&mut state.gc);
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
                OpCode::PushReference(pool_index) => {
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
                        .unwrap_or_else(|| {
                            panic!("Undeclared Variable \"{}\"", reference_name.as_str())
                        });
                    state.current_frame.op_stack.push(value);
                }
                OpCode::PushFunction(pool_index) => {
                    let rfunc = state.current_frame.bfunc.target.get_function(pool_index);
                    let bound_values = rfunc
                        .function
                        .binds
                        .iter()
                        .map(|name| {
                            state
                                .current_frame
                                .variables
                                .iter()
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
                                .unwrap_or_else(|| panic!("Undeclared Variable \"{}\"", name))
                        })
                        .collect();

                    let bfunc = rfunc.bind(bound_values, &mut state.gc);

                    state.current_frame.op_stack.push(Value::pusl_fn(bfunc));
                }
                OpCode::FunctionCall(num_args) => {
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
                            if new_frame.bfunc.target.function.is_generator {
                                let result = generator::new_generator(new_frame, &mut state);
                                state.current_frame.op_stack.push(result);
                            } else {
                                let old_frame =
                                    std::mem::replace(&mut state.current_frame, new_frame);
                                state.execution_stack.push(old_frame);
                            }
                        }
                        Value::Function((FunctionTarget::Native(handle), this)) => {
                            let this = this.map(|obj| Value::Object(obj));
                            let ptr = *state
                                .registry
                                .get(handle)
                                .expect("Out of bounds function handle");
                            native_fn_call = Some((ptr, args, this));
                        }
                        _ => panic!("Value must be a function to call"),
                    };
                }
                OpCode::FieldAccess(name_index) => {
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(name_index);
                    let value = match value {
                        Value::Object(object) => {
                            let value = object.deref().borrow().get_field(name.as_str());
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
                OpCode::Compare(op) => {
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
                    } else if let Some((path, parent_frame)) = state.resolve_stack.pop() {
                        let (frame, resolution) = {
                            let ExecutionState { imports, gc, .. } = &mut *state;
                            process_bcf(parent_frame, path, imports, gc)
                        };
                        state.current_frame = frame;
                        state.imports.push(resolution);
                        continue;
                    } else {
                        return Return(return_value);
                    }
                }
                OpCode::ConditionalJump(jump_index) => {
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
                OpCode::ComparisonJump(greater_index, less_index, equal_index) => {
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
                OpCode::Jump(jump_index) => {
                    state.current_frame.index = jump_index;
                }
                OpCode::Pop => {
                    state.current_frame.op_stack.pop().unwrap();
                }
                OpCode::IsNull => {
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let is_null = matches!(value, Value::Null);
                    state.current_frame.op_stack.push(Value::Boolean(is_null));
                }
                OpCode::Duplicate => {
                    let value = (*state.current_frame.op_stack.last().unwrap()).clone();
                    state.current_frame.op_stack.push(value);
                }
                OpCode::AssignReference(pool_index, is_let) => {
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
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
                        let variable_opt = state
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
                            .find(|var| var.name == reference_name);
                        let variable = match variable_opt {
                            Some(variable) => variable,
                            None => panic!(
                                "Cannot assign to non-existing variable {} without let",
                                reference_name
                            ),
                        };
                        variable.value = value;
                    }
                }
                OpCode::AssignField(pool_index, is_let) => {
                    let reference_name = state
                        .current_frame
                        .bfunc
                        .target
                        .function
                        .get_reference(pool_index);
                    let value = state.current_frame.op_stack.pop().unwrap();
                    let object = match state.current_frame.op_stack.pop().unwrap() {
                        Value::Object(ptr) => ptr,
                        other => panic!("Cannot Assign to field of {:?}", other),
                    };

                    if is_let {
                        (*object)
                            .borrow_mut()
                            .assign_field(reference_name.as_str(), value, true);
                    } else {
                        (*object)
                            .borrow_mut()
                            .assign_field(reference_name.as_str(), value, false);
                    }
                }
                OpCode::DuplicateMany(n) => {
                    let len = state.current_frame.op_stack.len();
                    assert!(n <= len);
                    let mut range = state.current_frame.op_stack[(len - n)..len].to_vec();
                    state.current_frame.op_stack.append(&mut range);
                }
                OpCode::PushBuiltin(pool_index) => {
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
                OpCode::DuplicateDeep(dup_index) => {
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
                    let result = state.current_frame.op_stack.pop().unwrap();
                    return Yield(result);
                }
                OpCode::Yeet => {
                    let error = state.current_frame.op_stack.pop().unwrap();
                    let error_obj = if let Value::Object(object_ptr) = error {
                        object_ptr
                    } else {
                        panic!("Can only yeet an object, not {:?}", error);
                    };
                    match unwind_stack(&mut state, current_idx, error_obj) {
                        Ok(error_catch) => current_catch = Some(error_catch),
                        Err(ret_val) => return ExecuteReturn::Error(ret_val),
                    }
                }
            }
        }
        if let Some((ptr, args, this)) = native_fn_call.take() {
            let result = ptr(args, this, st);
            st.borrow_mut().current_frame.op_stack.push(result);
        }
    }
}

fn unwind_stack(
    state: &mut ExecutionState,
    mut current_idx: usize,
    error: ObjectPtr,
) -> Result<(ObjectPtr, ErrorCatch), Value> {
    loop {
        for catch in &state.current_frame.bfunc.target.function.catches {
            if catch.begin <= current_idx && catch.filter > current_idx {
                state.current_frame.index = catch.filter;
                return Ok((error, catch.clone()));
            }
        }
        if let Some(mut new_frame) = state.execution_stack.pop() {
            mem::swap(&mut new_frame, &mut state.current_frame);
            current_idx = state.current_frame.index;
        } else {
            return Err(Value::Object(error));
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
    Value::Integer(lhs % rhs)
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
            Value::Null => matches!(rhs, Value::Null),
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
