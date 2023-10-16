use crate::backend::object::ObjectPtr;
use crate::backend::object::{FnPtr, PuslObject, Value};
use crate::backend::BoundFunction;
use crate::lexer::token::Literal;
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::{AssignAccess, AssignmentFlags};
use crate::parser::expression::{Compare, Expression};
use crate::parser::{Eval, ExpRef, Import, ParsedFile};
use core::num;
use garbage::ManagedPool;
use pad_adapter::PadAdapter;
use serde::{
    de::{Error, Visitor},
    ser::{self, SerializeSeq},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Write;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::{env, fmt};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(u64)]
// Top is Rhs (Bottom is lhs and calculated first)
pub enum OpCodeTag {
    Modulus,       // 2 Stack Values
    Literal,       // 1 ByteCode Value (index of literal pool)
    PushReference, // 1 ByteCode Value (index of reference pool)
    PushFunction,  //  1 ByteCode Value (index of sub-function pool) (also bind)
    PushThis,
    FunctionCall, // n + 1 Stack Values (bottom is reference to function) (first opcode is n)
    FieldAccess,  // 1 Stack value (object reference) and 1 ByteCode Value (index of reference pool)
    Addition,     // 2 Stack Values
    Subtraction,  // 2 Stack Values
    Negate,       // 2 Stack Values
    Multiply,     // 2 Stack Values
    Divide,       // 2 Stack Values
    AssignReference, // 1 Stack Value (value) and 2 opcode (reference, type)
    AssignField,  // 2 Stack Values (object - bottom, value - top) and 2 opcode (field name, type)
    DivideTruncate, // 2 Stack Values
    Exponent,     // 2 Stack Values
    Compare,      // 2 Stack Values (top is lhs), 1 OpCode (as Compare)
    And,          // 2 Stack Values
    Or,           // 2 Stack Values
    ScopeUp,      // Go into a new block
    ScopeDown,    // Leave a block
    Return,       // Return top of Stack
    ConditionalJump, // 1 Stack Value and 1 OpCode Value
    ComparisonJump, // 2 Stack Values and 3 OpCodes (Greater Than -> First Jump, Less Than -> Second Jump, Equal -> Third Jump)
    Jump,           // 1 OpCode Value
    Pop,            // Discard top value on stack
    IsNull,         // Replaces Value with False, Null with True
    Duplicate,      // Copies the top of the stack
    DuplicateMany,  // Copies n values onto top of stack (n is opcode)
    PushBuiltin,    // 1 ByteCode Value (index of reference pool)
    DuplicateDeep,  // 1 ByteCode Value (index of stack to duplicate (0 is top of stack))
    PushSelf,
    Yield, // Yield top of Stack
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    Modulus,              // 2 Stack Values
    Literal(usize),       // 1 ByteCode Value (index of literal pool)
    PushReference(usize), // 1 ByteCode Value (index of reference pool)
    PushFunction(usize),  //  1 ByteCode Value (index of sub-function pool) (also bind)
    PushThis,
    FunctionCall(usize), // n + 1 Stack Values (bottom is reference to function) (first opcode is n)
    FieldAccess(usize), // 1 Stack value (object reference) and 1 ByteCode Value (index of reference pool)
    Addition,           // 2 Stack Values
    Subtraction,        // 2 Stack Values
    Negate,             // 2 Stack Values
    Multiply,           // 2 Stack Values
    Divide,             // 2 Stack Values
    AssignReference(usize, bool), // 1 Stack Value (value) and 2 opcode (reference, type)
    AssignField(usize, bool), // 2 Stack Values (object - bottom, value - top) and 2 opcode (field name, type)
    DivideTruncate,           // 2 Stack Values
    Exponent,                 // 2 Stack Values
    Compare(Compare),         // 2 Stack Values (top is lhs), 1 OpCode (as Compare)
    And,                      // 2 Stack Values
    Or,                       // 2 Stack Values
    ScopeUp,                  // Go into a new block
    ScopeDown,                // Leave a block
    Return,                   // Return top of Stack
    ConditionalJump(usize),   // 1 Stack Value and 1 OpCode Value
    ComparisonJump(usize, usize, usize), // 2 Stack Values and 3 OpCodes (Greater Than -> First Jump, Less Than -> Second Jump, Equal -> Third Jump)
    Jump(usize),                         // 1 OpCode Value
    Pop,                                 // Discard top value on stack
    IsNull,                              // Replaces Value with False, Null with True
    Duplicate,                           // Copies the top of the stack
    DuplicateMany(usize),                // Copies n values onto top of stack (n is opcode)
    PushBuiltin(usize),                  // 1 ByteCode Value (index of reference pool)
    DuplicateDeep(usize), // 1 ByteCode Value (index of stack to duplicate (0 is top of stack))
    PushSelf,
    Yield, // Yield top of Stack
}

impl OpCode {
    pub fn format_opcode<W>(&self, index: usize, f: &mut W, func: &Function) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(f, "    {:3}; ", index)?;
        match *self {
            OpCode::PushThis => write!(f, "PushThis")?,
            OpCode::PushSelf => write!(f, "PushSelf")?,
            OpCode::Modulus => write!(f, "Modulus")?,
            OpCode::Literal(pool_index) => {
                let pool_value = &func.literals[pool_index];
                write!(f, "Literal {:?}[{}]", pool_value, pool_index)?;
            }
            OpCode::PushReference(pool_index) => {
                let pool_value = &func.references[pool_index];
                write!(f, "PushRef \"{}\"[{}]", pool_value, pool_index)?;
            }
            OpCode::PushFunction(pool_index) => {
                write!(f, "PushFunc [{}]", pool_index)?;
            }
            OpCode::FunctionCall(num_args) => {
                write!(f, "FnCall {}", num_args)?;
            }
            OpCode::FieldAccess(pool_index) => {
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
            OpCode::Compare(compare) => {
                write!(f, "Compare {:?}", compare)?;
            }
            OpCode::And => write!(f, "And")?,
            OpCode::Or => write!(f, "Or")?,
            OpCode::ScopeUp => write!(f, "ScopeUp")?,
            OpCode::ScopeDown => write!(f, "ScopeDown")?,
            OpCode::Return => write!(f, "Return")?,
            OpCode::ConditionalJump(jump_index) => {
                write!(f, "CndJmp {}", jump_index)?;
            }
            OpCode::ComparisonJump(greater_jump_index, less_jump_index, equal_jump_index) => {
                write!(
                    f,
                    "CmpJmp G:{} L:{} E:{}",
                    greater_jump_index, less_jump_index, equal_jump_index
                )?;
            }
            OpCode::Jump(jump_index) => {
                write!(f, "Jmp {}", jump_index)?;
            }
            OpCode::Pop => write!(f, "Pop")?,
            OpCode::IsNull => write!(f, "IsNull")?,
            OpCode::Duplicate => write!(f, "Duplicate")?,
            OpCode::AssignReference(pool_index, is_let) => {
                let pool_value = &func.references[pool_index];
                write!(
                    f,
                    "AssignRef let:{} \"{}\"[{}]",
                    is_let, pool_value, pool_index
                )?;
            }
            OpCode::AssignField(pool_index, is_let) => {
                let pool_value = &func.references[pool_index];
                write!(
                    f,
                    "AssignField let:{} \"{}\"[{}]",
                    is_let, pool_value, pool_index
                )?;
            }
            OpCode::DuplicateMany(n) => {
                write!(f, "DuplicateMany {}", n)?;
            }
            OpCode::PushBuiltin(pool_index) => {
                let pool_value = &func.references[pool_index];
                write!(f, "PushBuiltin \"{}\"[{}]", pool_value, pool_index)?;
            }
            OpCode::DuplicateDeep(dup_index) => {
                write!(f, "DuplicateDeep {}", dup_index)?;
            }
            OpCode::Yield => write!(f, "Yield")?,
        }

        Ok(())
    }
}

fn bytecode_decode(op_code: OpCodeTag) -> &'static [ByteCodeTag] {
    use ByteCodeTag::*;
    match op_code {
        OpCodeTag::Modulus => &[],
        OpCodeTag::Literal => &[Value],
        OpCodeTag::PushReference => &[Value],
        OpCodeTag::PushFunction => &[Value],
        OpCodeTag::PushThis => &[],
        OpCodeTag::FunctionCall => &[Value],
        OpCodeTag::FieldAccess => &[Value],
        OpCodeTag::Addition => &[],
        OpCodeTag::Subtraction => &[],
        OpCodeTag::Negate => &[],
        OpCodeTag::Multiply => &[],
        OpCodeTag::Divide => &[],
        OpCodeTag::AssignReference => &[Value, LetAssign],
        OpCodeTag::AssignField => &[Value, LetAssign],
        OpCodeTag::DivideTruncate => &[],
        OpCodeTag::Exponent => &[],
        OpCodeTag::Compare => &[Compare],
        OpCodeTag::And => &[],
        OpCodeTag::Or => &[],
        OpCodeTag::ScopeUp => &[],
        OpCodeTag::ScopeDown => &[],
        OpCodeTag::Return => &[],
        OpCodeTag::ConditionalJump => &[Value],
        OpCodeTag::ComparisonJump => &[Value, Value, Value],
        OpCodeTag::Jump => &[Value],
        OpCodeTag::Pop => &[],
        OpCodeTag::IsNull => &[],
        OpCodeTag::Duplicate => &[],
        OpCodeTag::DuplicateMany => &[Value],
        OpCodeTag::PushBuiltin => &[Value],
        OpCodeTag::DuplicateDeep => &[Value],
        OpCodeTag::PushSelf => &[],
        OpCodeTag::Yield => &[],
    }
}

enum ByteCodeTag {
    Value,
    Compare,
    LetAssign,
}

// TODO: I'm pretty sure this is bad
#[derive(Copy, Clone)]
#[repr(C)]
pub union ByteCode {
    op_code: OpCodeTag,
    value: u64,
    compare: Compare,
    let_assign: bool,
}

#[derive(Clone)]
pub struct ByteCodeArray(pub Vec<ByteCode>);

impl ByteCodeArray {
    pub fn get_offset(&self, offset: usize) -> Option<(OpCode, usize)> {
        let op_code_tag = self.0.get(offset).cloned().map(ByteCode::as_op);
        if let Some(op_code_tag) = op_code_tag {
            let (op_code, delta) = match op_code_tag {
                OpCodeTag::PushThis => (OpCode::PushThis, 1),
                OpCodeTag::PushSelf => (OpCode::PushSelf, 1),
                OpCodeTag::Modulus => (OpCode::Modulus, 1),
                OpCodeTag::Literal => {
                    let pool_index = self.0[offset + 1].as_val();
                    (OpCode::Literal(pool_index), 2)
                }
                OpCodeTag::PushReference => {
                    let pool_index = self.0[offset + 1].as_val();
                    (OpCode::PushReference(pool_index), 2)
                }
                OpCodeTag::PushFunction => {
                    let pool_index = self.0[offset + 1].as_val();
                    (OpCode::PushFunction(pool_index), 2)
                }
                OpCodeTag::FunctionCall => {
                    let num_args = self.0[offset + 1].as_val();
                    (OpCode::FunctionCall(num_args), 2)
                }
                OpCodeTag::FieldAccess => {
                    let pool_index = self.0[offset + 1].as_val();
                    (OpCode::FieldAccess(pool_index), 2)
                }
                OpCodeTag::Addition => (OpCode::Addition, 1),
                OpCodeTag::Subtraction => (OpCode::Subtraction, 1),
                OpCodeTag::Negate => (OpCode::Negate, 1),
                OpCodeTag::Multiply => (OpCode::Multiply, 1),
                OpCodeTag::Divide => (OpCode::Divide, 1),
                OpCodeTag::DivideTruncate => (OpCode::DivideTruncate, 1),
                OpCodeTag::Exponent => (OpCode::Exponent, 1),
                OpCodeTag::Compare => {
                    let compare = self.0[offset + 1].as_cmp();
                    (OpCode::Compare(compare), 2)
                }
                OpCodeTag::And => (OpCode::And, 1),
                OpCodeTag::Or => (OpCode::Or, 1),
                OpCodeTag::ScopeUp => (OpCode::ScopeUp, 1),
                OpCodeTag::ScopeDown => (OpCode::ScopeDown, 1),
                OpCodeTag::Return => (OpCode::Return, 1),
                OpCodeTag::ConditionalJump => {
                    let jump_index = self.0[offset + 1].as_val();
                    (OpCode::ConditionalJump(jump_index), 2)
                }
                OpCodeTag::ComparisonJump => {
                    let greater_jump_index = self.0[offset + 1].as_val();
                    let less_jump_index = self.0[offset + 2].as_val();
                    let equal_jump_index = self.0[offset + 3].as_val();
                    (
                        OpCode::ComparisonJump(
                            greater_jump_index,
                            less_jump_index,
                            equal_jump_index,
                        ),
                        4,
                    )
                }
                OpCodeTag::Jump => {
                    let jump_index = self.0[offset + 1].as_val();
                    (OpCode::Jump(jump_index), 2)
                }
                OpCodeTag::Pop => (OpCode::Pop, 1),
                OpCodeTag::IsNull => (OpCode::IsNull, 1),
                OpCodeTag::Duplicate => (OpCode::Duplicate, 1),
                OpCodeTag::AssignReference => {
                    let pool_index = self.0[offset + 1].as_val();
                    let is_let = self.0[offset + 2].as_bool();
                    (OpCode::AssignReference(pool_index, is_let), 3)
                }
                OpCodeTag::AssignField => {
                    let pool_index = self.0[offset + 1].as_val();
                    let is_let = self.0[offset + 2].as_bool();
                    (OpCode::AssignField(pool_index, is_let), 3)
                }
                OpCodeTag::DuplicateMany => {
                    let num = self.0[offset + 1].as_val();
                    (OpCode::DuplicateMany(num), 2)
                }
                OpCodeTag::PushBuiltin => {
                    let pool_index = self.0[offset + 1].as_val();
                    (OpCode::PushBuiltin(pool_index), 2)
                }
                OpCodeTag::DuplicateDeep => {
                    let dup_idx = self.0[offset + 1].as_val();
                    (OpCode::DuplicateDeep(dup_idx), 2)
                }
                OpCodeTag::Yield => (OpCode::Yield, 1),
            };

            Some((op_code, offset + delta))
        } else {
            None
        }
    }

    fn push(&mut self, op_code: OpCode) {
        match op_code {
            OpCode::Modulus => self.0.extend([ByteCode::op(OpCodeTag::Modulus)]),
            OpCode::Literal(idx) => self.0.extend([ByteCode::op(OpCodeTag::Literal), ByteCode::val(idx)]),
            OpCode::PushReference(idx) => self.0.extend([ByteCode::op(OpCodeTag::PushReference), ByteCode::val(idx)]),
            OpCode::PushFunction(idx) => self.0.extend([ByteCode::op(OpCodeTag::PushFunction), ByteCode::val(idx)]),
            OpCode::PushThis => self.0.extend([ByteCode::op(OpCodeTag::PushThis )]),
            OpCode::FunctionCall(idx) => self.0.extend([ByteCode::op(OpCodeTag::FunctionCall), ByteCode::val(idx)]),
            OpCode::FieldAccess(idx) => self.0.extend([ByteCode::op(OpCodeTag::FieldAccess), ByteCode::val(idx)]),
            OpCode::Addition => self.0.extend([ByteCode::op(OpCodeTag::Addition )]),
            OpCode::Subtraction => self.0.extend([ByteCode::op(OpCodeTag::Subtraction )]),
            OpCode::Negate => self.0.extend([ByteCode::op(OpCodeTag::Negate )]),
            OpCode::Multiply => self.0.extend([ByteCode::op(OpCodeTag::Multiply )]),
            OpCode::Divide => self.0.extend([ByteCode::op(OpCodeTag::Divide )]),
            OpCode::AssignReference(idx, is_let) => self.0.extend([ByteCode::op(OpCodeTag::AssignReference), ByteCode::val(idx), ByteCode::bool(is_let)]),
            OpCode::AssignField(idx, is_let) => self.0.extend([ByteCode::op(OpCodeTag::AssignField), ByteCode::val(idx), ByteCode::bool(is_let)]),
            OpCode::DivideTruncate => self.0.extend([ByteCode::op(OpCodeTag::DivideTruncate )]),
            OpCode::Exponent => self.0.extend([ByteCode::op(OpCodeTag::Exponent )]),
            OpCode::Compare(compare) => self.0.extend([ByteCode::op(OpCodeTag::Compare), ByteCode::cmp(compare)]),
            OpCode::And => self.0.extend([ByteCode::op(OpCodeTag::And )]),
            OpCode::Or => self.0.extend([ByteCode::op(OpCodeTag::Or )]),
            OpCode::ScopeUp => self.0.extend([ByteCode::op(OpCodeTag::ScopeUp )]),
            OpCode::ScopeDown => self.0.extend([ByteCode::op(OpCodeTag::ScopeDown )]),
            OpCode::Return => self.0.extend([ByteCode::op(OpCodeTag::Return )]),
            OpCode::ConditionalJump(offset) => self.0.extend([ByteCode::op(OpCodeTag::ConditionalJump), ByteCode::val(offset)]),
            OpCode::ComparisonJump(off1, off2, off3) => self.0.extend([ByteCode::op(OpCodeTag::ComparisonJump), ByteCode::val(off1), ByteCode::val(off2), ByteCode::val(off3)]),
            OpCode::Jump(offset) => self.0.extend([ByteCode::op(OpCodeTag::Jump), ByteCode::val(offset)]),
            OpCode::Pop => self.0.extend([ByteCode::op(OpCodeTag::Pop )]),
            OpCode::IsNull => self.0.extend([ByteCode::op(OpCodeTag::IsNull )]),
            OpCode::Duplicate => self.0.extend([ByteCode::op(OpCodeTag::Duplicate)]),
            OpCode::DuplicateMany(n) => self.0.extend([ByteCode::op(OpCodeTag::DuplicateMany), ByteCode::val(n)]),
            OpCode::PushBuiltin(n) => self.0.extend([ByteCode::op(OpCodeTag::PushBuiltin), ByteCode::val(n)]),
            OpCode::DuplicateDeep(n) => self.0.extend([ByteCode::op(OpCodeTag::DuplicateDeep), ByteCode::val(n)]),
            OpCode::PushSelf => self.0.extend([ByteCode::op(OpCodeTag::PushSelf)]),
            OpCode::Yield => self.0.extend([ByteCode::op(OpCodeTag::Yield)]),
        };

    }

    pub fn iter<'a>(&'a self) -> OpCodeIter<'a> {
        OpCodeIter {
            array: &self,
            offset: 0,
        }
    }
}

pub struct OpCodeIter<'a> {
    array: &'a ByteCodeArray,
    offset: usize,
}

impl<'a> Iterator for OpCodeIter<'a> {
    type Item = (usize, OpCode);

    fn next(&mut self) -> Option<Self::Item> {
        self.array
            .get_offset(self.offset)
            .map(|(result, new_offset)| {
                let old_offset = self.offset;
                self.offset = new_offset;
                (old_offset, result)
            })
    }
}

impl Serialize for ByteCodeArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut iter = self.0.iter();
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        while let Some(op_code) = iter.next() {
            let op_code = op_code.as_op();
            seq.serialize_element(&op_code)?;
            let following = bytecode_decode(op_code);
            let codes = iter.by_ref().take(following.len()).collect::<Vec<_>>();
            if following.len() != codes.len() {
                return Err(ser::Error::custom("Bytecode is an invalid length"));
            }
            for (tag, code) in following.into_iter().zip(codes) {
                unsafe {
                    match tag {
                        ByteCodeTag::Value => seq.serialize_element(&code.value)?,
                        ByteCodeTag::Compare => seq.serialize_element(&code.compare)?,
                        ByteCodeTag::LetAssign => seq.serialize_element(&code.let_assign)?,
                    }
                }
            }
        }

        seq.end()
    }
}

impl<'de> Deserialize<'de> for ByteCodeArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ByteCodeArrayVisitor)
    }
}

struct ByteCodeArrayVisitor;

impl<'de> Visitor<'de> for ByteCodeArrayVisitor {
    type Value = ByteCodeArray;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a pusl bytecode (8 bytes) representing an opcode or a u64"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut code = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        let err_fn = || A::Error::custom("Bytecode is an invalid length");
        while let Some(op_code) = seq.next_element::<OpCodeTag>()? {
            code.push(ByteCode::op(op_code));
            for tag in bytecode_decode(op_code) {
                match tag {
                    ByteCodeTag::Value => code.push(ByteCode::val(
                        seq.next_element::<u64>()?.ok_or_else(err_fn)? as usize,
                    )),
                    ByteCodeTag::Compare => code.push(ByteCode {
                        compare: seq.next_element::<Compare>()?.ok_or_else(err_fn)?,
                    }),
                    ByteCodeTag::LetAssign => code.push(ByteCode {
                        let_assign: seq.next_element::<bool>()?.ok_or_else(err_fn)?,
                    }),
                }
            }
        }

        Ok(ByteCodeArray(code))
    }
}

impl ByteCode {
    fn op(op_code: OpCodeTag) -> Self {
        ByteCode { op_code }
    }

    fn val(value: usize) -> Self {
        ByteCode {
            value: value as u64,
        }
    }

    fn bool(value: bool) -> Self {
        ByteCode {
            let_assign: value
        }
    }

    fn cmp(value: Compare) -> Self {
        ByteCode {
            compare: value
        }
    }


    fn zero() -> Self {
        ByteCode { value: 0 }
    }

    fn as_op(self) -> OpCodeTag {
        unsafe { self.op_code }
    }

    fn as_val(self) -> usize {
        unsafe { self.value as usize }
    }

    fn as_cmp(self) -> Compare {
        unsafe { self.compare }
    }

    fn as_bool(self) -> bool {
        unsafe { self.let_assign }
    }

    unsafe fn from_u64(v: u64) -> Self {
        std::mem::transmute::<u64, Self>(v)
    }

    pub unsafe fn to_u64(&self) -> u64 {
        std::mem::transmute_copy::<Self, u64>(self)
    }
}

#[derive(Serialize, Deserialize)]
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
                    "\t{:3}; {} as {}",
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
                writeln!(f, "\n+{:->4}:", index)?;
                let mut adapter = PadAdapter::with_padding(f, "|   ");
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

    pub fn bind(&'static self, bound_values: Vec<Value>, gc: &mut ManagedPool) -> FnPtr {
        let bfunc = BoundFunction {
            target: self,
            bound_values,
        };
        gc.place_in_heap(bfunc)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub binds: Vec<String>,
    pub literals: Vec<Literal>,
    pub references: Vec<String>,
    pub code: ByteCodeArray,
    pub is_generator: bool,
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
                // TODO: Should calculate padding needed for the intr. idx
                writeln!(f, "\n+{:->4}:", index)?;
                let mut adapter = PadAdapter::with_padding(f, "|   ");
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
        gc: &mut ManagedPool,
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
                .find(|i| i.0 == path)
                .map(|i| i.1.clone())
                .unwrap();
            let import_object = PuslObject::new_with_parent(import_parent);
            let import_ptr = gc.place_in_heap(import_object) as ObjectPtr;
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
                self.code.0.len(),
            )?;
        } else {
            writeln!(f, "\nLiterals:")?;
            for (index, literal) in self.literals.iter().enumerate() {
                writeln!(f, "    {:3}; {:?}", index, literal)?;
            }
            writeln!(f, "References:")?;
            for (index, reference) in self.references.iter().enumerate() {
                writeln!(f, "    {:3}; {}", index, reference)?;
            }
            writeln!(f, "Code:")?;
            let mut code_iter = self.code.iter().peekable();

            while let Some((idx, op_code)) = code_iter.next() {
                op_code.format_opcode(idx, f, self)?;
                if code_iter.peek().is_some() {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

impl Function {
    pub fn get_code(&self, index: usize) -> Option<(OpCode, usize)> {
        self.code.get_offset(index)
    }

    pub fn get_val(&self, index: usize) -> usize {
        self.code.0[index].as_val()
    }

    pub fn get_cmp(&self, index: usize) -> Compare {
        self.code.0[index].as_cmp()
    }

    pub fn get_assign_type(&self, index: usize) -> bool {
        self.code.0[index].as_bool()
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
        self.code.0[index].value = jump_to as u64;
    }

    fn place_jump(&mut self, conditional: bool) -> usize {
        let op = if conditional {
            OpCodeTag::ConditionalJump
        } else {
            OpCodeTag::Jump
        };
        self.code.0.push(ByteCode::op(op));
        let index = self.current_index();
        self.code.0.push(ByteCode::zero());
        index
    }

    fn place_jump_to(&mut self, conditional: bool, jump_to: usize) {
        let op = if conditional {
            OpCodeTag::ConditionalJump
        } else {
            OpCodeTag::Jump
        };
        self.code.0.push(ByteCode::op(op));
        self.code.0.push(ByteCode::val(jump_to));
    }

    fn current_index(&self) -> usize {
        self.code.0.len()
    }

    fn new(args: Vec<String>, binds: Vec<String>) -> Function {
        Function {
            args,
            binds,
            literals: vec![],
            references: vec![],
            code: ByteCodeArray(vec![]),
            is_generator: false,
        }
    }
}

pub fn linearize_file(file: ParsedFile, path: PathBuf) -> ByteCodeFile {
    let ParsedFile { expr, imports } = file;
    let func = linearize(expr, vec![], vec![]);
    let bcf = ByteCodeFile {
        file: path,
        base_func: func,
        imports,
    };
    if env::var("PUSL_TRACE_CODE").is_ok() {
        println!("Code:\n{:#?}", &bcf)
    }
    bcf
}

fn linearize(expr: ExpRef, args: Vec<String>, binds: Vec<String>) -> BasicFunction {
    let code = Function::new(args, binds);
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
            func.function.code.0.push(ByteCode::op(OpCodeTag::Modulus));
            true
        }
        Expression::Literal { value } => {
            let literal_index = func.function.add_literal(value);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Literal));
            func.function.code.0.push(ByteCode::val(literal_index));
            true
        }
        Expression::ThisReference => {
            func.function.code.0.push(ByteCode::op(OpCodeTag::PushThis));
            true
        }
        Expression::Reference { target } => {
            let reference_index = func.function.add_reference(target);
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::PushReference));
            func.function.code.0.push(ByteCode::val(reference_index));
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
            linearize_exp_ref(target, func, true);
            let num_args = arguments.len();
            arguments
                .into_iter()
                .for_each(|argument| linearize_exp_ref(argument, func, true));
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::FunctionCall));
            func.function.code.0.push(ByteCode::val(num_args));
            true
        }
        Expression::FieldAccess { target, name } => {
            linearize_exp_ref(target, func, true);
            let reference_index = func.function.add_reference(name);
            func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
            func.function.code.0.push(ByteCode::val(reference_index));
            true
        }
        Expression::Addition { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Addition));
            true
        }
        Expression::Subtract { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Subtraction));
            true
        }
        Expression::Negate { operand } => {
            linearize_exp_ref(operand, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
            true
        }
        Expression::Multiply { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Multiply));
            true
        }
        Expression::Divide { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Multiply));
            true
        }
        Expression::Elvis { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Duplicate));
            func.function.code.0.push(ByteCode::op(OpCodeTag::IsNull));
            func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
            let use_first_index = func.function.place_jump(true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Pop));
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
                        func.function.code.0.push(ByteCode::op(OpCodeTag::Duplicate));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
                        func.function.code.0.push(ByteCode::val(target_index));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::IsNull));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function.code.0.push(ByteCode::op(OpCodeTag::AssignField));
                    func.function.code.0.push(ByteCode::val(target_index));
                    func.function.code.0.push(ByteCode {
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
                        func.function
                            .code
                            .0
                            .push(ByteCode::op(OpCodeTag::PushReference));
                        func.function.code.0.push(ByteCode::val(target_index));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::IsNull));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function
                        .code
                        .0
                        .push(ByteCode::op(OpCodeTag::AssignReference));
                    func.function.code.0.push(ByteCode::val(target_index));
                    func.function.code.0.push(ByteCode {
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
                        func.function
                            .code
                            .0
                            .push(ByteCode::op(OpCodeTag::DuplicateDeep));
                        func.function.code.0.push(ByteCode::val(1));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
                        let pool_index = func.function.add_reference(String::from("@index_get"));
                        func.function.code.0.push(ByteCode::val(pool_index));
                        func.function
                            .code
                            .0
                            .push(ByteCode::op(OpCodeTag::DuplicateDeep));
                        func.function.code.0.push(ByteCode::val(1));
                        func.function
                            .code
                            .0
                            .push(ByteCode::op(OpCodeTag::FunctionCall));
                        func.function.code.0.push(ByteCode::val(1));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::IsNull));
                        func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
                        Some(func.function.place_jump(true))
                    } else {
                        None
                    };
                    func.function
                        .code
                        .0
                        .push(ByteCode::op(OpCodeTag::DuplicateDeep));
                    func.function.code.0.push(ByteCode::val(1));
                    func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
                    let pool_index = func.function.add_reference(String::from("@index_set"));
                    func.function.code.0.push(ByteCode::val(pool_index));
                    func.function
                        .code
                        .0
                        .push(ByteCode::op(OpCodeTag::DuplicateDeep));
                    func.function.code.0.push(ByteCode::val(1));
                    linearize_exp_ref(expression, func, true);
                    func.function
                        .code
                        .0
                        .push(ByteCode::op(OpCodeTag::FunctionCall));
                    func.function.code.0.push(ByteCode::val(2));
                    if let Some(jump_instruction) = skip_index {
                        func.function
                            .set_jump(jump_instruction, func.function.current_index());
                    }

                    func.function.code.0.push(ByteCode::op(OpCodeTag::Pop));
                    func.function.code.0.push(ByteCode::op(OpCodeTag::Pop));
                }
            };

            false
        }
        Expression::DivideTruncate { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::DivideTruncate));
            true
        }
        Expression::Exponent { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Exponent));
            true
        }
        Expression::Compare {
            lhs,
            rhs,
            operation,
        } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Compare));
            func.function.code.0.push(ByteCode { compare: operation });
            true
        }
        Expression::And { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::And));
            true
        }
        Expression::Or { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Or));
            true
        }
        Expression::FunctionDeclaration {
            binds,
            params,
            body,
        } => {
            let new_func = linearize(body, params, binds);
            let index = func.sub_functions.len();
            func.sub_functions.push(new_func);
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::PushFunction));
            func.function.code.0.push(ByteCode::val(index));
            true
        }
        Expression::Return { value } => {
            linearize_exp_ref(value, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Return));
            false
        }
        Expression::ListDeclaration { values } => {
            // TODO: Create List Object
            func.function.code.0.push(ByteCode::op(OpCodeTag::PushBuiltin));
            let pool_index = func.function.add_reference(String::from("List"));
            func.function.code.0.push(ByteCode::val(pool_index));
            let num_values = values.len();
            values
                .into_iter()
                .for_each(|value| linearize_exp_ref(value, func, true));
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::FunctionCall));
            func.function.code.0.push(ByteCode::val(num_values));
            true
        }
        Expression::ListAccess { target, index } => {
            linearize_exp_ref(target, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
            let pool_index = func.function.add_reference(String::from("@index_get"));
            func.function.code.0.push(ByteCode::val(pool_index));
            linearize_exp_ref(index, func, true);
            func.function
                .code
                .0
                .push(ByteCode::op(OpCodeTag::FunctionCall));
            func.function.code.0.push(ByteCode::val(1));
            true
        }
        Expression::SelfReference => {
            func.function.code.0.push(ByteCode::op(OpCodeTag::PushSelf));
            true
        }
        Expression::Yield { value } => {
            func.function.is_generator = true;
            linearize_exp_ref(value, func, true);
            func.function.code.0.push(ByteCode::op(OpCodeTag::Yield));
            false
        }
    };
    match (expand_stack, created_value) {
        (true, false) => panic!(),
        (false, true) => func.function.code.0.push(ByteCode::op(OpCodeTag::Pop)),
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
        Branch::ForLoop {
            variable,
            iterable,
            body,
        } => linearize_for(variable, iterable, body, func),
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
        .0
        .push(ByteCode::op(OpCodeTag::ComparisonJump));
    let jump_table = func.function.current_index();
    func.function.code.0.push(ByteCode::zero());
    func.function.code.0.push(ByteCode::zero());
    func.function.code.0.push(ByteCode::zero());
    let indexes = body
        .into_iter()
        .map(|expr| {
            let start_index = func.function.current_index();
            func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeUp));
            linearize_exp_ref(expr, func, false);
            func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeDown));
            let jump_out_index = func.function.place_jump(false);
            (start_index, jump_out_index)
        })
        .collect::<Vec<_>>();
    func.function.code.0[jump_table + 0].value = indexes[greater as usize].0 as u64;
    func.function.code.0[jump_table + 1].value = indexes[less as usize].0 as u64;
    func.function.code.0[jump_table + 2].value = indexes[equal as usize].0 as u64;
    let jump_out_to = func.function.current_index();
    indexes.into_iter().for_each(|(_, jump_out_index)| {
        func.function.code.0[jump_out_index].value = jump_out_to as u64
    });
}

fn linearize_for(variable: String, iterable: ExpRef, body: ExpRef, func: &mut BasicFunction) {
    linearize_exp_ref(iterable, func, true);
    let condition_idx = func.function.current_index();
    func.function.code.0.push(ByteCode::op(OpCodeTag::Duplicate));
    let has_next_reference = func.function.add_reference("hasNext".to_string());
    func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
    func.function.code.0.push(ByteCode::val(has_next_reference));
    func.function
        .code
        .0
        .push(ByteCode::op(OpCodeTag::FunctionCall));
    func.function.code.0.push(ByteCode::zero());
    func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
    let store_loop_end_idx = func.function.place_jump(true);
    func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeUp));
    // AssignReference, // 1 Stack Value (value) and 2 opcode (reference, type)
    func.function.code.0.push(ByteCode::op(OpCodeTag::Duplicate));
    let next_reference = func.function.add_reference("next".to_string());
    func.function.code.0.push(ByteCode::op(OpCodeTag::FieldAccess));
    func.function.code.0.push(ByteCode::val(next_reference));
    func.function
        .code
        .0
        .push(ByteCode::op(OpCodeTag::FunctionCall));
    func.function.code.0.push(ByteCode::zero());

    let target_idx = func.function.add_reference(variable);
    func.function
        .code
        .0
        .push(ByteCode::op(OpCodeTag::AssignReference));
    func.function.code.0.push(ByteCode::val(target_idx));
    func.function.code.0.push(ByteCode { let_assign: true });
    linearize_exp_ref(body, func, false);
    func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeDown));
    func.function.place_jump_to(false, condition_idx);

    func.function
        .set_jump(store_loop_end_idx, func.function.current_index());
    func.function.code.0.push(ByteCode::op(OpCodeTag::Pop));
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
        func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeUp));
        linearize_exp_ref(else_expr, func, false);
        func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeDown));
    }
    let jump_to_end_index = func.function.place_jump(false);
    let place_bodies = place_conditions
        .into_iter()
        .map(|(jump_index, body)| {
            let jump_to = func.function.current_index();
            func.function.set_jump(jump_index, jump_to);
            func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeUp));
            linearize_exp_ref(body, func, false);
            func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeDown));
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
    func.function.code.0.push(ByteCode::op(OpCodeTag::Negate));
    let condition_jump_index = func.function.place_jump(true);
    func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeUp));
    linearize_exp_ref(body, func, false);
    func.function.code.0.push(ByteCode::op(OpCodeTag::ScopeDown));
    func.function.place_jump_to(false, begin_index);
    let end_index = func.function.current_index();
    func.function.set_jump(condition_jump_index, end_index);
}
