use std::fmt;

use serde::{Serialize, Deserialize, ser::{self, SerializeSeq}, Serializer, Deserializer, de::Visitor, de::Error};

use crate::parser::expression::Compare;

use super::linearize::Function;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(u64)]
// Top is Rhs (Bottom is lhs and calculated first)
enum OpCodeTag {
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
    Yeet,  // Yeet top of Stack
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
    Yeet,  // Yeet top of Stack
}

impl OpCode {
    pub fn jump(is_conditional: bool, target: usize) -> OpCode {
        match is_conditional {
            true => OpCode::ConditionalJump(target),
            false => OpCode::Jump(target),
        }
    }
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
            OpCode::Yeet => write!(f, "Yeet")?,
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
        OpCodeTag::Yeet => &[],
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
union ByteCode {
    op_code: OpCodeTag,
    value: u64,
    compare: Compare,
    let_assign: bool,
}

#[derive(Clone)]
pub struct ByteCodeArray(Vec<ByteCode>);

impl ByteCodeArray {
    // TODO: For safety, only accept a wrapper around usize as an offset
    pub fn new() -> Self {
        ByteCodeArray(Vec::new())
    }
    pub fn get(&self, offset: usize) -> Option<(OpCode, usize)> {
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
                OpCodeTag::Yeet => (OpCode::Yeet, 1),
            };

            Some((op_code, offset + delta))
        } else {
            None
        }
    }

    pub fn push(&mut self, op_code: OpCode) {
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
            OpCode::Yeet => self.0.extend([ByteCode::op(OpCodeTag::Yeet)]),
        };

    }

    pub fn iter<'a>(&'a self) -> OpCodeIter<'a> {
        OpCodeIter {
            array: &self,
            offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn place_jump(&mut self, conditional: bool) -> impl FnOnce(&mut Self, Option<usize>) {
        let op = if conditional {
            OpCode::ConditionalJump(0)
        } else {
            OpCode::Jump(0)
        };
        self.push(op);
        let jump_target_loc = self.0.len() - 1;
        return move |this, jump_target| {
            this.0[jump_target_loc] = ByteCode::val(jump_target.unwrap_or(this.len()));
        }
    }

    pub fn place_cmp_jump(&mut self) -> impl FnOnce(&mut Self, [usize; 3]) {
        self.push(OpCode::ComparisonJump(0, 0, 0));
        let jump_target_arr_loc = self.0.len() - 3;
        return move |this, jump_target| {
            for i in 0..3 {
                this.0[jump_target_arr_loc + i] = ByteCode::val(jump_target[i]);
            }
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
            .get(self.offset)
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

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn as_bool(self) -> bool {
        unsafe { self.let_assign }
    }

    fn as_cmp(self) -> Compare {
        unsafe { self.compare }
    }

}
