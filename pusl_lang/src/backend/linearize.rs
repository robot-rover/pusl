use crate::lexer::token::Literal;
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::{AssignmentFlags, AssignAccess};
use crate::parser::expression::{Compare, Expression};
use crate::parser::{Eval, ExpRef, Import, ParsedFile};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, io};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use crate::backend::RFunction;
use crate::backend::object::{ObjectPtr, Object};
use garbage::ManagedPool;
use std::cell::RefCell;
use std::thread::LocalKey;

#[derive(Copy, Clone, Debug)]
// Top is Rhs (Bottom is lhs and calculated first)
pub enum OpCode {
    Modulus,         // 2 Stack Values
    Literal,         // 1 ByteCode Value (index of literal pool)
    PushReference,   // 1 ByteCode Value (index of reference pool)
    PushFunction,    //  1 ByteCode Value (index of sub-function pool)
    PushSelf,
    FunctionCall,    // n + 1 Stack Values (bottom is reference to function) (first opcode is n)
    MethodCall, // n + 2 Stack Values (bottom is self, then function, then n arguments) first opcode is n
    FieldAccess, // 1 Stack value (object reference) and 1 ByteCode Value (index of reference pool)
    Addition,   // 2 Stack Values
    Subtraction, // 2 Stack Values
    Negate,     // 2 Stack Values
    Multiply,   // 2 Stack Values
    Divide,     // 2 Stack Values
    AssignReference, // 1 Stack Value (value) and 2 opcode (reference, type)
    AssignField, // 2 Stack Values (object - bottom, value - top) and 2 opcode (field name, type)
    DivideTruncate, // 2 Stack Values
    Exponent,   // 2 Stack Values
    Compare,    // 2 Stack Values (top is lhs), 1 OpCode (as Compare)
    And,        // 2 Stack Values
    Or,         // 2 Stack Values
    ScopeUp,    // Go into a new block
    ScopeDown,  // Leave a block
    Return,     // Return top of Stack
    ConditionalJump, // 1 Stack Value and 1 OpCode Value
    ComparisonJump, // 2 Stack Values and 3 OpCodes (Greater Than -> First Jump, Less Than -> Second Jump, Equal -> Third Jump)
    Jump,           // 1 OpCode Value
    Pop,            // Discard top value on stack
    IsNull,         // Replaces Value with False, Null with True
    Duplicate,      // Copies the top of the stack
    DuplicateMany,  // Copies n values onto top of stack (n is opcode)
    ListAccess,    // 2 Stack Values Bottom: ArrayReference, Top: ArrayIndex
    AssignList,    // 3 Stack Values Bottom: ArrayReference, Array Index, Top: Value and 1 opcode (type)
    NewList
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union ByteCode {
    op_code: OpCode,
    value: u64,
    compare: Compare,
    let_assign: bool,
}

impl Serialize for ByteCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(unsafe { self.value })
    }
}

impl<'de> Deserialize<'de> for ByteCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(ByteCodeVisitor)
    }
}

struct ByteCodeVisitor;

impl<'de> Visitor<'de> for ByteCodeVisitor {
    type Value = ByteCode;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a pusl bytecode (8 bytes) representing an opcode or a u64"
        )
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ByteCode { value: v })
    }
}

impl ByteCode {
    pub fn op(op_code: OpCode) -> Self {
        ByteCode { op_code }
    }

    pub fn val(value: usize) -> Self {
        ByteCode {
            value: value as u64,
        }
    }

    pub fn zero() -> Self {
        ByteCode { value: 0 }
    }

    pub fn as_op(self) -> OpCode {
        unsafe { self.op_code }
    }

    pub fn as_val(self) -> usize {
        unsafe { self.value as usize }
    }

    pub fn as_cmp(self) -> Compare {
        unsafe { self.compare }
    }

    pub fn as_bool(self) -> bool {
        unsafe { self.let_assign }
    }
}

pub struct ByteCodeFile {
    pub file: PathBuf,
    pub base_func: Function<()>,
    pub imports: Vec<Import>,
}

impl Debug for ByteCodeFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            write!(f, "ByteCode: {}, Imports: {}, {:?}", self.file.display(), self.imports.len(), self.base_func)?;
        } else {
            writeln!(f, "ByteCode: {}", self.file.display())?;
            writeln!(f, "Imports:")?;
            for (index, import) in self.imports.iter().enumerate() {
                writeln!(f, "\t{:3}: {} as {}", index, import.path.display(), import.alias)?;
            }
            write!(f, "{:#?}", self.base_func)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Function<T> {
    pub args: Vec<String>,
    literals: Vec<Literal>,
    references: Vec<String>,
    pub(crate) code: Vec<ByteCode>,
    pub sub_functions: Vec<Function<T>>,
    #[serde(skip)]
    pub resolved: T
}

pub fn resolve<'a, I>(function: Function<()>, global_imports: I, target_imports: Vec<Import>, gc: &'static LocalKey<RefCell<ManagedPool>>) -> &'static RFunction
    where I: IntoIterator<Item = &'a (PathBuf, ObjectPtr)>{
    let Function { args, literals, references, code, sub_functions, .. } = function;

    let mut iter = global_imports.into_iter();
    let mut imports = Vec::new();
    for Import { path, alias } in target_imports {
        let import_parent: ObjectPtr = iter.by_ref().find(|i| &i.0 == &path).map(|i| i.1.clone()).unwrap();
        let import_object = Object::new_with_parent(import_parent);
        let import_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(import_object));
        imports.push((alias, import_ptr));
    }

    let imports: &Vec<_> = Box::leak(Box::new(imports));

    let sub_functions = sub_functions.into_iter().map(|f| sub_resolve(f, imports)).collect();

    let result = RFunction {
        args,
        literals,
        references,
        code,
        sub_functions,
        resolved: imports
    };
    Box::leak(Box::new(result))
}

fn sub_resolve(function: Function<()>, imports: &'static Vec<(String, ObjectPtr)>) -> RFunction {
    let Function { args, literals, references, code, sub_functions, .. } = function;
    let sub_functions = sub_functions.into_iter().map(|f| sub_resolve(f, imports)).collect();
    RFunction {
        args,
        literals,
        references,
        code,
        sub_functions,
        resolved: imports
    }
}

impl<T> Debug for Function<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Function(")?;
        let mut arg_iter = self.args.iter().peekable();
        while let Some(arg_name) = arg_iter.next() {
            write!(f, "{}", arg_name)?;
            if arg_iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;
        if !f.alternate() {
            write!(
                f,
                " - lits: {}, refs: {}, code: {}, sub-funcs: {}",
                self.literals.len(),
                self.references.len(),
                self.code.len(),
                self.sub_functions.len()
            )?;
        } else {
            writeln!(f, "\nLiterals:")?;
            for (index, literal) in self.literals.iter().enumerate() {
                writeln!(f, "\t{:3}: {:?}", index, literal)?;
            }
            writeln!(f, "References:")?;
            for (index, reference) in self.references.iter().enumerate() {
                writeln!(f, "\t{:3}: {}", index, reference)?;
            }
            writeln!(f, "Sub-Functions:")?;
            for (index, sub_function) in self.sub_functions.iter().enumerate() {
                writeln!(f, "\t{:3}: {:?}", index, sub_function)?;
            }
            writeln!(f, "Code:")?;
            let mut code_iter = self.code.iter().enumerate();
            while let Some(tuple) = code_iter.next() {
                write_bytecode_line(tuple, f, &mut code_iter, &self)?;
            }
        }
        Ok(())
    }
}

pub fn write_bytecode_line<'a, T, F, W>(
    line: (usize, &ByteCode),
    f: &mut W,
    code_iter: &mut T,
    func: &Function<F>,
) -> fmt::Result
where
    T: Iterator<Item = (usize, &'a ByteCode)>,
    W: fmt::Write
{
    let (index, bytecode) = line;
    let op_code = bytecode.as_op();
    write!(f, "\t{:3}: ", index)?;
    match op_code {
        OpCode::PushSelf => writeln!(f, "PushSelf")?,
        OpCode::Modulus => writeln!(f, "Modulus")?,
        OpCode::Literal => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.literals[pool_index];
            writeln!(f, "Literal {:?}[{}]", pool_value, pool_index)?;
        }
        OpCode::PushReference => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            writeln!(f, "PushRef \"{}\"[{}]", pool_value, pool_index)?;
        }
        OpCode::PushFunction => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.sub_functions[pool_index];
            writeln!(f, "PushFunc {:?} | [{}]", pool_value, pool_index)?;
        }
        OpCode::FunctionCall => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            writeln!(f, "FnCall {}", pool_index)?;
        }
        OpCode::MethodCall => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            writeln!(f, "ObjCall {}", pool_index)?;
        }
        OpCode::FieldAccess => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            writeln!(f, "Field {}[{}]", pool_value, pool_index)?;
        }
        OpCode::Addition => writeln!(f, "Addition")?,
        OpCode::Subtraction => writeln!(f, "Subtraction")?,
        OpCode::Negate => writeln!(f, "Negate")?,
        OpCode::Multiply => writeln!(f, "Multiply")?,
        OpCode::Divide => writeln!(f, "Divide")?,
        OpCode::DivideTruncate => writeln!(f, "DivTrunc")?,
        OpCode::Exponent => writeln!(f, "Exponent")?,
        OpCode::Compare => {
            let compare = unsafe { code_iter.next().unwrap().1.compare };
            writeln!(f, "Compare {:?}", compare)?;
        }
        OpCode::And => writeln!(f, "And")?,
        OpCode::Or => writeln!(f, "Or")?,
        OpCode::ScopeUp => writeln!(f, "ScopeUp")?,
        OpCode::ScopeDown => writeln!(f, "ScopeDown")?,
        OpCode::Return => writeln!(f, "Return")?,
        OpCode::ConditionalJump => {
            let jump_index = code_iter.next().unwrap().1.as_val();
            writeln!(f, "CndJmp {}", jump_index)?;
        }
        OpCode::ComparisonJump => {
            let greater_jump_index = code_iter.next().unwrap().1.as_val();
            let less_jump_index = code_iter.next().unwrap().1.as_val();
            let equal_jump_index = code_iter.next().unwrap().1.as_val();
            writeln!(
                f,
                "CmpJmp G:{} L:{} E:{}",
                greater_jump_index, less_jump_index, equal_jump_index
            )?;
        }
        OpCode::Jump => {
            let jump_index = code_iter.next().unwrap().1.as_val();
            writeln!(f, "Jmp {}", jump_index)?;
        }
        OpCode::Pop => writeln!(f, "Pop")?,
        OpCode::IsNull => writeln!(f, "IsNull")?,
        OpCode::Duplicate => writeln!(f, "Duplicate")?,
        OpCode::AssignReference => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
            writeln!(
                f,
                "AssignRef let:{} \"{}\"[{}]",
                is_let, pool_value, pool_index
            )?;
        }
        OpCode::AssignField => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
            writeln!(
                f,
                "AssignField let:{} \"{}\"[{}]",
                is_let, pool_value, pool_index
            )?;
        }
        OpCode::DuplicateMany => {
            let n = code_iter.next().unwrap().1.as_val();
            writeln!(f, "DuplicateMany {}", n)?;
        }
        OpCode::ListAccess => {
            writeln!(f, "ListAccess")?;
        }
        OpCode::AssignList => {
            let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
            writeln!(f, "AssignList let:{}", is_let)?;
        }
        OpCode::NewList => {
            writeln!(f, "NewList")?;
        }
    }

    Ok(())
}

impl<T> Function<T> {

    pub fn get_code(&self, index: usize) -> Option<OpCode> {
        self.code.get(index).map(|b| b.as_op())
    }

    pub fn get_val(&self, index: usize) -> usize {
        self.code[index].as_val()
    }

    pub fn get_cmp(&self, index: usize) -> Compare {
        self.code[index].as_cmp()
    }

    pub fn get_assign_type(&self, index: usize) -> bool {
        self.code[index].as_bool()
    }

    //TODO: don't clone this
    pub fn get_literal(&self, index: usize) -> Literal {
        self.literals[index].clone()
    }

    //TODO: don't clone this
    pub fn get_reference(&self, index: usize) -> String {
        self.references[index].clone()
    }

    pub fn get_function(&self, index: usize) -> &Function<T> {
        &self.sub_functions[index]
    }

    fn add_literal(&mut self, literal: Literal) -> usize {
        let exists = self
            .literals
            .iter()
            .enumerate()
            .find(|(_, existing)| &&literal == existing)
            .map(|(index, _)| index);
        exists.unwrap_or_else(|| {
            let index = self.literals.len();
            self.literals.push(literal);
            index
        })
    }

    fn add_reference(&mut self, reference: String) -> usize {
        let exists = self
            .references
            .iter()
            .enumerate()
            .find(|(_, existing)| &&reference == existing)
            .map(|(index, _)| index);
        exists.unwrap_or_else(|| {
            let index = self.references.len();
            self.references.push(reference);
            index
        })
    }

    fn set_jump(&mut self, index: usize, jump_to: usize) {
        self.code[index].value = jump_to as u64;
    }

    fn place_jump(&mut self, conditional: bool) -> usize {
        let op = if conditional {
            OpCode::ConditionalJump
        } else {
            OpCode::Jump
        };
        self.code.push(ByteCode::op(op));
        let index = self.current_index();
        self.code.push(ByteCode::zero());
        index
    }

    fn place_jump_to(&mut self, conditional: bool, jump_to: usize) {
        let op = if conditional {
            OpCode::ConditionalJump
        } else {
            OpCode::Jump
        };
        self.code.push(ByteCode::op(op));
        self.code.push(ByteCode::val(jump_to));
    }

    fn current_index(&self) -> usize {
        self.code.len()
    }

    fn with_args(args: Vec<String>, resolved: T) -> Function<T> {
        Function {
            args,
            literals: vec![],
            references: vec![],
            code: vec![],
            sub_functions: vec![],
            resolved
        }
    }
}

pub fn linearize_file(file: ParsedFile, path: PathBuf) -> ByteCodeFile {
    let ParsedFile { expr, imports } = file;
    let func = linearize(expr, vec![]);
    ByteCodeFile { file: path, base_func: func, imports }
}

fn linearize(expr: ExpRef, args: Vec<String>) -> Function<()> {
    let mut code = Function::<()>::with_args(args, ());
    linearize_exp_ref(expr, &mut code, false);

    code
}

fn linearize_exp_ref(exp_ref: ExpRef, func: &mut Function<()>, expand_stack: bool) {
    match *exp_ref {
        Eval::Branch(branch) => {
            assert!(!expand_stack);
            linearize_branch(branch, func)
        }
        Eval::Expression(expr) => linearize_expr(expr, func, expand_stack),
    }
}

fn linearize_expr(expr: Expression, func: &mut Function<()>, expand_stack: bool) {
    let created_value = match expr {
        Expression::Modulus { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Modulus));
            true
        }
        Expression::Literal { value } => {
            let literal_index = func.add_literal(value);
            func.code.push(ByteCode::op(OpCode::Literal));
            func.code.push(ByteCode::val(literal_index));
            true
        }
        Expression::SelfReference => {
            func.code.push(ByteCode::op(OpCode::PushSelf));
            true
        }
        Expression::Reference { target } => {
            let reference_index = func.add_reference(target);
            func.code.push(ByteCode::op(OpCode::PushReference));
            func.code.push(ByteCode::val(reference_index));
            true
        }
        Expression::Joiner { expressions } => {
            assert!(!expand_stack);
            expressions
                .into_iter()
                .for_each(|expr| linearize_exp_ref(expr, func, false));
            false
        }
        Expression::FunctionCall { target, arguments } => {
            func.code.push(ByteCode::op(OpCode::PushReference));
            let pool_index = func.add_reference(target);
            func.code.push(ByteCode::val(pool_index));
            let num_args = arguments.len();
            arguments
                .into_iter()
                .for_each(|argument| linearize_exp_ref(argument, func, true));
            func.code.push(ByteCode::op(OpCode::FunctionCall));
            func.code.push(ByteCode::val(num_args));
            true
        }
        Expression::MethodCall {
            target,
            field,
            arguments,
        } => {
            linearize_exp_ref(target, func, true);
            func.code.push(ByteCode::op(OpCode::Duplicate));
            func.code.push(ByteCode::op(OpCode::FieldAccess));
            let pool_index = func.add_reference(field);
            func.code.push(ByteCode::val(pool_index));
            let num_args = arguments.len();
            arguments
                .into_iter()
                .for_each(|argument| linearize_exp_ref(argument, func, true));
            func.code.push(ByteCode::op(OpCode::MethodCall));
            func.code.push(ByteCode::val(num_args));
            true
        }
        Expression::FieldAccess { target, name } => {
            linearize_exp_ref(target, func, true);
            let reference_index = func.add_reference(name);
            func.code.push(ByteCode::op(OpCode::FieldAccess));
            func.code.push(ByteCode::val(reference_index));
            true
        }
        Expression::Addition { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Addition));
            true
        }
        Expression::Subtract { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Subtraction));
            true
        }
        Expression::Negate { operand } => {
            linearize_exp_ref(operand, func, true);
            func.code.push(ByteCode::op(OpCode::Negate));
            true
        }
        Expression::Multiply { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Multiply));
            true
        }
        Expression::Divide { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Multiply));
            true
        }
        Expression::Elvis { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            func.code.push(ByteCode::op(OpCode::Duplicate));
            func.code.push(ByteCode::op(OpCode::IsNull));
            func.code.push(ByteCode::op(OpCode::Negate));
            let use_first_index = func.place_jump(true);
            func.code.push(ByteCode::op(OpCode::Pop));
            linearize_exp_ref(rhs, func, true);
            let current_index = func.current_index();
            func.set_jump(use_first_index, current_index);
            true
        }
        Expression::Assigment {
            target,
            expression,
            flags,
        } => {
            let skip_index_option = match target {
                AssignAccess::Field { target, name } => {
                    linearize_exp_ref(target, func, true);
                    let target_index = func.add_reference(name);
                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.code.push(ByteCode::op(OpCode::Duplicate));
                        func.code.push(ByteCode::op(OpCode::FieldAccess));
                        func.code.push(ByteCode::val(target_index));
                        func.code.push(ByteCode::op(OpCode::IsNull));
                        func.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.code.push(ByteCode::op(OpCode::AssignField));
                    func.code.push(ByteCode::val(target_index));
                    skip_index
                },
                AssignAccess::Reference { name } => {
                    let target_index = func.add_reference(name);
                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.code.push(ByteCode::op(OpCode::PushReference));
                        func.code.push(ByteCode::val(target_index));
                        func.code.push(ByteCode::op(OpCode::IsNull));
                        func.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.code.push(ByteCode::op(OpCode::AssignReference));
                    func.code.push(ByteCode::val(target_index));
                    skip_index
                },
                AssignAccess::Array { target, index } => {
                    linearize_exp_ref(target, func, true);
                    linearize_exp_ref(index, func, true);
                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.code.push(ByteCode::op(OpCode::DuplicateMany));
                        func.code.push(ByteCode::val(2));
                        func.code.push(ByteCode::op(OpCode::ListAccess));
                        func.code.push(ByteCode::op(OpCode::IsNull));
                        func.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.code.push(ByteCode::op(OpCode::AssignList));
                    skip_index
                },
            };


            func.code.push(ByteCode {
                let_assign: flags.intersects(AssignmentFlags::LET),
            });
            if let Some(jump_instruction) = skip_index_option {
                func.set_jump(jump_instruction, func.current_index());
            }
            false
        }
        Expression::DivideTruncate { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::DivideTruncate));
            true
        }
        Expression::Exponent { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Exponent));
            true
        }
        Expression::Compare {
            lhs,
            rhs,
            operation,
        } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Compare));
            func.code.push(ByteCode { compare: operation });
            true
        }
        Expression::And { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::And));
            true
        }
        Expression::Or { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::Or));
            true
        }
        Expression::FunctionDeclaration { params, body } => {
            let new_func = linearize(body, params);
            let index = func.sub_functions.len();
            func.sub_functions.push(new_func);
            func.code.push(ByteCode::op(OpCode::PushFunction));
            func.code.push(ByteCode::val(index));
            true
        }
        Expression::Return { value } => {
            linearize_exp_ref(value, func, true);
            func.code.push(ByteCode::op(OpCode::Return));
            false
        }
        Expression::ListDeclaration { values } => {
            func.code.push(ByteCode::op(OpCode::NewList));
            for value in values {
                func.code.push(ByteCode::op(OpCode::Duplicate));
                func.code.push(ByteCode::op(OpCode::Duplicate));
                func.code.push(ByteCode::op(OpCode::FieldAccess));
                let pool_index = func.add_reference(String::from("push"));
                func.code.push(ByteCode::val(pool_index));
                linearize_exp_ref(value, func, true);
                func.code.push(ByteCode::op(OpCode::MethodCall));
                func.code.push(ByteCode::val(1));
                func.code.push(ByteCode::op(OpCode::Pop))

            }
            true
        }
        Expression::ListAccess { target, index } => {
            linearize_exp_ref(target, func, true);
            linearize_exp_ref(index, func, true);
            func.code.push(ByteCode::op(OpCode::ListAccess));
            true
        }
    };
    match (expand_stack, created_value) {
        (true, false) => panic!(),
        (false, true) => func.code.push(ByteCode::op(OpCode::Pop)),
        _ => {}
    }
}

fn linearize_branch(branch: Branch, func: &mut Function<()>) {
    match branch {
        Branch::WhileLoop { condition, body } => linearize_while(condition, body, func),
        Branch::IfElseBlock { conditions, last } => linearize_if_else(conditions, last, func),
        Branch::CompareBlock {
            lhs,
            rhs,
            greater,
            equal,
            less,
            body,
        } => linearize_compare(lhs, rhs, greater, equal, less, body, func),
        _ => panic!(),
    }
}

fn linearize_compare(
    lhs: ExpRef,
    rhs: ExpRef,
    greater: u8,
    equal: u8,
    less: u8,
    body: Vec<ExpRef>,
    func: &mut Function<()>,
) {
    linearize_exp_ref(lhs, func, true);
    linearize_exp_ref(rhs, func, true);
    func.code.push(ByteCode::op(OpCode::ComparisonJump));
    let jump_table = func.current_index();
    func.code.push(ByteCode::zero());
    func.code.push(ByteCode::zero());
    func.code.push(ByteCode::zero());
    let indexes = body
        .into_iter()
        .map(|expr| {
            let start_index = func.current_index();
            func.code.push(ByteCode::op(OpCode::ScopeUp));
            linearize_exp_ref(expr, func, false);
            func.code.push(ByteCode::op(OpCode::ScopeDown));
            let jump_out_index = func.place_jump(false);
            (start_index, jump_out_index)
        })
        .collect::<Vec<_>>();
    func.code[jump_table + 0].value = indexes[greater as usize].0 as u64;
    func.code[jump_table + 1].value = indexes[less as usize].0 as u64;
    func.code[jump_table + 2].value = indexes[equal as usize].0 as u64;
    let jump_out_to = func.current_index();
    indexes
        .into_iter()
        .for_each(|(_, jump_out_index)| func.code[jump_out_index].value = jump_out_to as u64);
}

fn linearize_if_else(conditions: Vec<ConditionBody>, last: Option<ExpRef>, func: &mut Function<()>) {
    let place_conditions = conditions
        .into_iter()
        .map(|ConditionBody { condition, body }| {
            linearize_exp_ref(condition, func, true);
            let jump_index = func.place_jump(true);
            (jump_index, body)
        })
        .collect::<Vec<_>>();
    if let Some(else_expr) = last {
        func.code.push(ByteCode::op(OpCode::ScopeUp));
        linearize_exp_ref(else_expr, func, false);
        func.code.push(ByteCode::op(OpCode::ScopeDown));
    }
    let jump_to_end_index = func.place_jump(false);
    let place_bodies = place_conditions
        .into_iter()
        .map(|(jump_index, body)| {
            let jump_to = func.current_index();
            func.set_jump(jump_index, jump_to);
            func.code.push(ByteCode::op(OpCode::ScopeUp));
            linearize_exp_ref(body, func, false);
            func.code.push(ByteCode::op(OpCode::ScopeDown));
            let jump_to_end_index = func.place_jump(false);
            jump_to_end_index
        })
        .collect::<Vec<_>>();
    let jump_to = func.current_index();
    place_bodies.into_iter().for_each(|jump_index| {
        func.set_jump(jump_index, jump_to);
    });
    func.set_jump(jump_to_end_index, jump_to);
}

fn linearize_while(condition: ExpRef, body: ExpRef, func: &mut Function<()>) {
    let begin_index = func.current_index();
    linearize_exp_ref(condition, func, true);
    func.code.push(ByteCode::op(OpCode::Negate));
    let condition_jump_index = func.place_jump(true);
    func.code.push(ByteCode::op(OpCode::ScopeUp));
    linearize_exp_ref(body, func, false);
    func.code.push(ByteCode::op(OpCode::ScopeDown));
    func.place_jump_to(false, begin_index);
    let end_index = func.current_index();
    func.set_jump(condition_jump_index, end_index);
}
