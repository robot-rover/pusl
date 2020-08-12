use crate::backend::object::{Object, ObjectPtr};
use crate::lexer::token::Literal;
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::{AssignAccess, AssignmentFlags};
use crate::parser::expression::{Compare, Expression};
use crate::parser::{Eval, ExpRef, Import, ParsedFile};
use garbage::ManagedPool;
use pad_adapter::PadAdapter;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::fmt;
use std::fmt::Write;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::thread::LocalKey;

#[derive(Copy, Clone, Debug)]
// Top is Rhs (Bottom is lhs and calculated first)
pub enum OpCode {
    Modulus,       // 2 Stack Values
    Literal,       // 1 ByteCode Value (index of literal pool)
    PushReference, // 1 ByteCode Value (index of reference pool)
    PushFunction,  //  1 ByteCode Value (index of sub-function pool)
    PushSelf,
    FunctionCall, // n + 1 Stack Values (bottom is reference to function) (first opcode is n)
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
    PushBuiltin,    // 1 ByteCode Value (index of reference pool)
    DuplicateDeep,  // 1 ByteCode Value (index of stack to duplicate (0 is top of stack))
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
    pub base_func: BasicFunction,
    pub imports: Vec<Import>,
}

impl Debug for ByteCodeFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            write!(
                f,
                "ByteCode: {}, Imports: {}, {:?}",
                self.file.display(),
                self.imports.len(),
                self.base_func
            )?;
        } else {
            writeln!(f, "ByteCode: {}", self.file.display())?;
            writeln!(f, "Imports:")?;
            for (index, import) in self.imports.iter().enumerate() {
                writeln!(
                    f,
                    "\t{:3}: {} as {}",
                    index,
                    import.path.display(),
                    import.alias
                )?;
            }
            write!(f, "{:#?}", self.base_func)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ResolvedFunction {
    pub function: Function,
    pub imports: &'static Vec<(String, ObjectPtr)>,
    pub sub_functions: Vec<ResolvedFunction>,
}

impl Debug for ResolvedFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            self.function.fmt(f)?;
            write!(
                f,
                ", sub-fns: {}, imports: {}",
                self.sub_functions.len(),
                self.imports.len()
            )
        } else {
            self.function.fmt(f)?;
            write!(f, "\nImports:")?;
            for (import, target) in self.imports {
                write!(f, "\n{} => {:?}", import, target)?;
            }
            write!(f, "\nSub-Functions:")?;
            for (index, sub_fn) in self.sub_functions.iter().enumerate() {
                write!(f, "\n{:4}:", index)?;
                let mut adapter = PadAdapter::new(f);
                write!(adapter, "{:#?}", sub_fn)?;
            }
            Ok(())
        }
    }
}

impl ResolvedFunction {
    pub fn get_function(&self, pool_index: usize) -> &ResolvedFunction {
        &self.sub_functions[pool_index]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Function {
    pub args: Vec<String>,
    literals: Vec<Literal>,
    references: Vec<String>,
    pub(crate) code: Vec<ByteCode>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BasicFunction {
    pub function: Function,
    pub sub_functions: Vec<BasicFunction>,
}

impl Debug for BasicFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            self.function.fmt(f)?;
            write!(f, ", sub-fns: {}", self.sub_functions.len())
        } else {
            self.function.fmt(f)?;
            write!(f, "\nSub-Functions:")?;
            for (index, sub_fn) in self.sub_functions.iter().enumerate() {
                writeln!(f, "\n{:4}:", index)?;
                let mut adapter = PadAdapter::new(f);
                write!(adapter, "{:#?}", sub_fn)?;
            }
            Ok(())
        }
    }
}

impl From<Function> for BasicFunction {
    fn from(function: Function) -> Self {
        BasicFunction {
            function,
            sub_functions: Vec::new(),
        }
    }
}

impl BasicFunction {
    pub fn resolve<'a, I>(
        self,
        global_imports: I,
        target_imports: Vec<Import>,
        gc: &'static LocalKey<RefCell<ManagedPool>>,
    ) -> &'static ResolvedFunction
    where
        I: IntoIterator<Item = &'a (PathBuf, ObjectPtr)>,
    {
        let BasicFunction {
            function,
            sub_functions,
        } = self;
        let mut iter = global_imports.into_iter();
        let mut imports = Vec::new();
        for Import { path, alias } in target_imports {
            let import_parent: ObjectPtr = iter
                .by_ref()
                .find(|i| &i.0 == &path)
                .map(|i| i.1.clone())
                .unwrap();
            let import_object = Object::new_with_parent(import_parent);
            let import_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(import_object));
            imports.push((alias, import_ptr));
        }

        let imports: &Vec<_> = Box::leak(Box::new(imports));

        let sub_functions = sub_functions
            .into_iter()
            .map(|f| f.sub_resolve(imports))
            .collect();

        let result = ResolvedFunction {
            function,
            imports,
            sub_functions,
        };
        Box::leak(Box::new(result))
    }

    fn sub_resolve(self, imports: &'static Vec<(String, ObjectPtr)>) -> ResolvedFunction {
        let BasicFunction {
            function,
            sub_functions,
        } = self;
        let sub_functions = sub_functions
            .into_iter()
            .map(|f| f.sub_resolve(imports))
            .collect();
        ResolvedFunction {
            function,
            sub_functions,
            imports,
        }
    }
}

impl Debug for Function {
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
                " - lits: {}, refs: {}, code: {}",
                self.literals.len(),
                self.references.len(),
                self.code.len(),
            )?;
        } else {
            writeln!(f, "\nLiterals:")?;
            for (index, literal) in self.literals.iter().enumerate() {
                writeln!(f, "    {:3}: {:?}", index, literal)?;
            }
            writeln!(f, "References:")?;
            for (index, reference) in self.references.iter().enumerate() {
                writeln!(f, "    {:3}: {}", index, reference)?;
            }
            writeln!(f, "Code:")?;
            let mut code_iter = self.code.iter().enumerate().peekable();
            while let Some(tuple) = code_iter.next() {
                write_bytecode_line(tuple, f, &mut code_iter, &self)?;
                if code_iter.peek().is_some() {
                    writeln!(f, "")?;
                }
            }
        }
        Ok(())
    }
}

pub fn write_bytecode_line<'a, T, W>(
    line: (usize, &ByteCode),
    f: &mut W,
    code_iter: &mut T,
    func: &Function,
) -> fmt::Result
where
    T: Iterator<Item = (usize, &'a ByteCode)>,
    W: fmt::Write,
{
    let (index, bytecode) = line;
    let op_code = bytecode.as_op();
    write!(f, "    {:3}: ", index)?;
    match op_code {
        OpCode::PushSelf => write!(f, "PushSelf")?,
        OpCode::Modulus => write!(f, "Modulus")?,
        OpCode::Literal => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.literals[pool_index];
            write!(f, "Literal {:?}[{}]", pool_value, pool_index)?;
        }
        OpCode::PushReference => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            write!(f, "PushRef \"{}\"[{}]", pool_value, pool_index)?;
        }
        OpCode::PushFunction => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            write!(f, "PushFunc [{}]", pool_index)?;
        }
        OpCode::FunctionCall => {
            let num_args = code_iter.next().unwrap().1.as_val();
            write!(f, "FnCall {}", num_args)?;
        }
        OpCode::MethodCall => {
            let num_args = code_iter.next().unwrap().1.as_val();
            write!(f, "MethodCall {}", num_args)?;
        }
        OpCode::FieldAccess => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            write!(f, "Field {}[{}]", pool_value, pool_index)?;
        }
        OpCode::Addition => write!(f, "Addition")?,
        OpCode::Subtraction => write!(f, "Subtraction")?,
        OpCode::Negate => write!(f, "Negate")?,
        OpCode::Multiply => write!(f, "Multiply")?,
        OpCode::Divide => write!(f, "Divide")?,
        OpCode::DivideTruncate => write!(f, "DivTrunc")?,
        OpCode::Exponent => write!(f, "Exponent")?,
        OpCode::Compare => {
            let compare = unsafe { code_iter.next().unwrap().1.compare };
            write!(f, "Compare {:?}", compare)?;
        }
        OpCode::And => write!(f, "And")?,
        OpCode::Or => write!(f, "Or")?,
        OpCode::ScopeUp => write!(f, "ScopeUp")?,
        OpCode::ScopeDown => write!(f, "ScopeDown")?,
        OpCode::Return => write!(f, "Return")?,
        OpCode::ConditionalJump => {
            let jump_index = code_iter.next().unwrap().1.as_val();
            write!(f, "CndJmp {}", jump_index)?;
        }
        OpCode::ComparisonJump => {
            let greater_jump_index = code_iter.next().unwrap().1.as_val();
            let less_jump_index = code_iter.next().unwrap().1.as_val();
            let equal_jump_index = code_iter.next().unwrap().1.as_val();
            write!(
                f,
                "CmpJmp G:{} L:{} E:{}",
                greater_jump_index, less_jump_index, equal_jump_index
            )?;
        }
        OpCode::Jump => {
            let jump_index = code_iter.next().unwrap().1.as_val();
            write!(f, "Jmp {}", jump_index)?;
        }
        OpCode::Pop => write!(f, "Pop")?,
        OpCode::IsNull => write!(f, "IsNull")?,
        OpCode::Duplicate => write!(f, "Duplicate")?,
        OpCode::AssignReference => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
            write!(
                f,
                "AssignRef let:{} \"{}\"[{}]",
                is_let, pool_value, pool_index
            )?;
        }
        OpCode::AssignField => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
            write!(
                f,
                "AssignField let:{} \"{}\"[{}]",
                is_let, pool_value, pool_index
            )?;
        }
        OpCode::DuplicateMany => {
            let n = code_iter.next().unwrap().1.as_val();
            write!(f, "DuplicateMany {}", n)?;
        }
        OpCode::PushBuiltin => {
            let pool_index = code_iter.next().unwrap().1.as_val();
            let pool_value = &func.references[pool_index];
            write!(f, "PushBuiltin \"{}\"[{}]", pool_value, pool_index)?;
        }
        OpCode::DuplicateDeep => {
            let dup_index = code_iter.next().unwrap().1.as_val();
            write!(f, "DuplicateDeep {}", dup_index)?;
        }
    }

    Ok(())
}

impl Function {
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

    fn new(args: Vec<String>) -> Self {
        Function {
            args,
            literals: vec![],
            references: vec![],
            code: vec![],
        }
    }
}

pub fn linearize_file(file: ParsedFile, path: PathBuf) -> ByteCodeFile {
    let ParsedFile { expr, imports } = file;
    let func = linearize(expr, vec![]);
    ByteCodeFile {
        file: path,
        base_func: func,
        imports,
    }
}

fn linearize(expr: ExpRef, args: Vec<String>) -> BasicFunction {
    let code = Function::new(args);
    let mut function = BasicFunction::from(code);
    linearize_exp_ref(expr, &mut function, false);

    function
}

fn linearize_exp_ref(exp_ref: ExpRef, func: &mut BasicFunction, expand_stack: bool) {
    match *exp_ref {
        Eval::Branch(branch) => {
            assert!(!expand_stack);
            linearize_branch(branch, func)
        }
        Eval::Expression(expr) => linearize_expr(expr, func, expand_stack),
    }
}

fn linearize_expr(expr: Expression, func: &mut BasicFunction, expand_stack: bool) {
    let created_value = match expr {
        Expression::Modulus { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Modulus));
            true
        }
        Expression::Literal { value } => {
            let literal_index = func.function.add_literal(value);
            func.function.code.push(ByteCode::op(OpCode::Literal));
            func.function.code.push(ByteCode::val(literal_index));
            true
        }
        Expression::SelfReference => {
            func.function.code.push(ByteCode::op(OpCode::PushSelf));
            true
        }
        Expression::Reference { target } => {
            let reference_index = func.function.add_reference(target);
            func.function.code.push(ByteCode::op(OpCode::PushReference));
            func.function.code.push(ByteCode::val(reference_index));
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
            func.function.code.push(ByteCode::op(OpCode::PushReference));
            let pool_index = func.function.add_reference(target);
            func.function.code.push(ByteCode::val(pool_index));
            let num_args = arguments.len();
            arguments
                .into_iter()
                .for_each(|argument| linearize_exp_ref(argument, func, true));
            func.function.code.push(ByteCode::op(OpCode::FunctionCall));
            func.function.code.push(ByteCode::val(num_args));
            true
        }
        Expression::MethodCall {
            target,
            field,
            arguments,
        } => {
            linearize_exp_ref(target, func, true);
            func.function.code.push(ByteCode::op(OpCode::Duplicate));
            func.function.code.push(ByteCode::op(OpCode::FieldAccess));
            let pool_index = func.function.add_reference(field);
            func.function.code.push(ByteCode::val(pool_index));
            let num_args = arguments.len();
            arguments
                .into_iter()
                .for_each(|argument| linearize_exp_ref(argument, func, true));
            func.function.code.push(ByteCode::op(OpCode::MethodCall));
            func.function.code.push(ByteCode::val(num_args));
            true
        }
        Expression::FieldAccess { target, name } => {
            linearize_exp_ref(target, func, true);
            let reference_index = func.function.add_reference(name);
            func.function.code.push(ByteCode::op(OpCode::FieldAccess));
            func.function.code.push(ByteCode::val(reference_index));
            true
        }
        Expression::Addition { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Addition));
            true
        }
        Expression::Subtract { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Subtraction));
            true
        }
        Expression::Negate { operand } => {
            linearize_exp_ref(operand, func, true);
            func.function.code.push(ByteCode::op(OpCode::Negate));
            true
        }
        Expression::Multiply { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Multiply));
            true
        }
        Expression::Divide { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Multiply));
            true
        }
        Expression::Elvis { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Duplicate));
            func.function.code.push(ByteCode::op(OpCode::IsNull));
            func.function.code.push(ByteCode::op(OpCode::Negate));
            let use_first_index = func.function.place_jump(true);
            func.function.code.push(ByteCode::op(OpCode::Pop));
            linearize_exp_ref(rhs, func, true);
            let current_index = func.function.current_index();
            func.function.set_jump(use_first_index, current_index);
            true
        }
        Expression::Assigment {
            target,
            expression,
            flags,
        } => {
            match target {
                AssignAccess::Field { target, name } => {
                    linearize_exp_ref(target, func, true);
                    let target_index = func.function.add_reference(name);
                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function.code.push(ByteCode::op(OpCode::Duplicate));
                        func.function.code.push(ByteCode::op(OpCode::FieldAccess));
                        func.function.code.push(ByteCode::val(target_index));
                        func.function.code.push(ByteCode::op(OpCode::IsNull));
                        func.function.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function.code.push(ByteCode::op(OpCode::AssignField));
                    func.function.code.push(ByteCode::val(target_index));
                    func.function.code.push(ByteCode {
                        let_assign: flags.intersects(AssignmentFlags::LET),
                    });
                    if let Some(jump_instruction) = skip_index {
                        func.function
                            .set_jump(jump_instruction, func.function.current_index());
                    }
                }
                AssignAccess::Reference { name } => {
                    let target_index = func.function.add_reference(name);
                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function.code.push(ByteCode::op(OpCode::PushReference));
                        func.function.code.push(ByteCode::val(target_index));
                        func.function.code.push(ByteCode::op(OpCode::IsNull));
                        func.function.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function
                        .code
                        .push(ByteCode::op(OpCode::AssignReference));
                    func.function.code.push(ByteCode::val(target_index));
                    func.function.code.push(ByteCode {
                        let_assign: flags.intersects(AssignmentFlags::LET),
                    });
                    if let Some(jump_instruction) = skip_index {
                        func.function
                            .set_jump(jump_instruction, func.function.current_index());
                    }
                }
                AssignAccess::Array { target, index } => {
                    linearize_exp_ref(target, func, true);
                    linearize_exp_ref(index, func, true);

                    let skip_index = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function.code.push(ByteCode::op(OpCode::DuplicateDeep));
                        func.function.code.push(ByteCode::val(1));
                        func.function.code.push(ByteCode::op(OpCode::Duplicate));
                        func.function.code.push(ByteCode::op(OpCode::FieldAccess));
                        let pool_index = func.function.add_reference(String::from("@index_get"));
                        func.function.code.push(ByteCode::val(pool_index));
                        func.function.code.push(ByteCode::op(OpCode::DuplicateDeep));
                        func.function.code.push(ByteCode::val(2));
                        func.function.code.push(ByteCode::op(OpCode::MethodCall));
                        func.function.code.push(ByteCode::val(1));
                        func.function.code.push(ByteCode::op(OpCode::IsNull));
                        func.function.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    func.function.code.push(ByteCode::op(OpCode::DuplicateDeep));
                    func.function.code.push(ByteCode::val(1));
                    func.function.code.push(ByteCode::op(OpCode::FieldAccess));
                    let pool_index = func.function.add_reference(String::from("@index_set"));
                    func.function.code.push(ByteCode::val(pool_index));
                    func.function.code.push(ByteCode::op(OpCode::DuplicateDeep));
                    func.function.code.push(ByteCode::val(2));
                    linearize_exp_ref(expression, func, true);
                    func.function.code.push(ByteCode::op(OpCode::MethodCall));
                    func.function.code.push(ByteCode::val(2));
                    if let Some(jump_instruction) = skip_index {
                        func.function
                            .set_jump(jump_instruction, func.function.current_index());
                    }

                    func.function.code.push(ByteCode::op(OpCode::Pop));
                    func.function.code.push(ByteCode::op(OpCode::Pop));
                }
            };

            false
        }
        Expression::DivideTruncate { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function
                .code
                .push(ByteCode::op(OpCode::DivideTruncate));
            true
        }
        Expression::Exponent { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Exponent));
            true
        }
        Expression::Compare {
            lhs,
            rhs,
            operation,
        } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Compare));
            func.function.code.push(ByteCode { compare: operation });
            true
        }
        Expression::And { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::And));
            true
        }
        Expression::Or { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(ByteCode::op(OpCode::Or));
            true
        }
        Expression::FunctionDeclaration { params, body } => {
            let new_func = linearize(body, params);
            let index = func.sub_functions.len();
            func.sub_functions.push(new_func);
            func.function.code.push(ByteCode::op(OpCode::PushFunction));
            func.function.code.push(ByteCode::val(index));
            true
        }
        Expression::Return { value } => {
            linearize_exp_ref(value, func, true);
            func.function.code.push(ByteCode::op(OpCode::Return));
            false
        }
        Expression::ListDeclaration { values } => {
            // TODO: Create List Object
            func.function.code.push(ByteCode::op(OpCode::PushBuiltin));
            let pool_index = func.function.add_reference(String::from("List"));
            func.function.code.push(ByteCode::val(pool_index));
            let num_values = values.len();
            values
                .into_iter()
                .for_each(|value| linearize_exp_ref(value, func, true));
            func.function.code.push(ByteCode::op(OpCode::FunctionCall));
            func.function.code.push(ByteCode::val(num_values));
            true
        }
        Expression::ListAccess { target, index } => {
            linearize_exp_ref(target, func, true);
            func.function.code.push(ByteCode::op(OpCode::Duplicate));
            func.function.code.push(ByteCode::op(OpCode::FieldAccess));
            let pool_index = func.function.add_reference(String::from("@index_get"));
            func.function.code.push(ByteCode::val(pool_index));
            linearize_exp_ref(index, func, true);
            func.function.code.push(ByteCode::op(OpCode::MethodCall));
            func.function.code.push(ByteCode::val(1));
            true
        }
    };
    match (expand_stack, created_value) {
        (true, false) => panic!(),
        (false, true) => func.function.code.push(ByteCode::op(OpCode::Pop)),
        _ => {}
    }
}

fn linearize_branch(branch: Branch, func: &mut BasicFunction) {
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
    func: &mut BasicFunction,
) {
    linearize_exp_ref(lhs, func, true);
    linearize_exp_ref(rhs, func, true);
    func.function
        .code
        .push(ByteCode::op(OpCode::ComparisonJump));
    let jump_table = func.function.current_index();
    func.function.code.push(ByteCode::zero());
    func.function.code.push(ByteCode::zero());
    func.function.code.push(ByteCode::zero());
    let indexes = body
        .into_iter()
        .map(|expr| {
            let start_index = func.function.current_index();
            func.function.code.push(ByteCode::op(OpCode::ScopeUp));
            linearize_exp_ref(expr, func, false);
            func.function.code.push(ByteCode::op(OpCode::ScopeDown));
            let jump_out_index = func.function.place_jump(false);
            (start_index, jump_out_index)
        })
        .collect::<Vec<_>>();
    func.function.code[jump_table + 0].value = indexes[greater as usize].0 as u64;
    func.function.code[jump_table + 1].value = indexes[less as usize].0 as u64;
    func.function.code[jump_table + 2].value = indexes[equal as usize].0 as u64;
    let jump_out_to = func.function.current_index();
    indexes.into_iter().for_each(|(_, jump_out_index)| {
        func.function.code[jump_out_index].value = jump_out_to as u64
    });
}

fn linearize_if_else(
    conditions: Vec<ConditionBody>,
    last: Option<ExpRef>,
    func: &mut BasicFunction,
) {
    let place_conditions = conditions
        .into_iter()
        .map(|ConditionBody { condition, body }| {
            linearize_exp_ref(condition, func, true);
            let jump_index = func.function.place_jump(true);
            (jump_index, body)
        })
        .collect::<Vec<_>>();
    if let Some(else_expr) = last {
        func.function.code.push(ByteCode::op(OpCode::ScopeUp));
        linearize_exp_ref(else_expr, func, false);
        func.function.code.push(ByteCode::op(OpCode::ScopeDown));
    }
    let jump_to_end_index = func.function.place_jump(false);
    let place_bodies = place_conditions
        .into_iter()
        .map(|(jump_index, body)| {
            let jump_to = func.function.current_index();
            func.function.set_jump(jump_index, jump_to);
            func.function.code.push(ByteCode::op(OpCode::ScopeUp));
            linearize_exp_ref(body, func, false);
            func.function.code.push(ByteCode::op(OpCode::ScopeDown));
            let jump_to_end_index = func.function.place_jump(false);
            jump_to_end_index
        })
        .collect::<Vec<_>>();
    let jump_to = func.function.current_index();
    place_bodies.into_iter().for_each(|jump_index| {
        func.function.set_jump(jump_index, jump_to);
    });
    func.function.set_jump(jump_to_end_index, jump_to);
}

fn linearize_while(condition: ExpRef, body: ExpRef, func: &mut BasicFunction) {
    let begin_index = func.function.current_index();
    linearize_exp_ref(condition, func, true);
    func.function.code.push(ByteCode::op(OpCode::Negate));
    let condition_jump_index = func.function.place_jump(true);
    func.function.code.push(ByteCode::op(OpCode::ScopeUp));
    linearize_exp_ref(body, func, false);
    func.function.code.push(ByteCode::op(OpCode::ScopeDown));
    func.function.place_jump_to(false, begin_index);
    let end_index = func.function.current_index();
    func.function.set_jump(condition_jump_index, end_index);
}
