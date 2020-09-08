use crate::lexer::token::Literal;
use crate::parser::ExpRef;

#[derive(Debug)]
pub enum AssignAccess {
    Field { target: ExpRef, name: String },

    Reference { name: String },

    Array { target: ExpRef, index: ExpRef },
}

/// Syntax Blocks which are linear
/// i.e. they will never branch
#[derive(Debug)]
pub enum Expression {
    Modulus {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Literal {
        value: Literal,
    },

    SelfReference,

    Reference {
        target: String,
    },

    Joiner {
        expressions: Vec<ExpRef>,
    },

    FunctionCall {
        target: String,
        arguments: Vec<ExpRef>,
    },

    MethodCall {
        target: ExpRef,
        field: String,
        arguments: Vec<ExpRef>,
    },

    FieldAccess {
        target: ExpRef,
        name: String,
    },

    Addition {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Subtract {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    /// Double Duty, negate numbers and binary not
    Negate {
        operand: ExpRef,
    },

    Multiply {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Divide {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Elvis {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Assigment {
        target: AssignAccess,
        expression: ExpRef,
        flags: AssignmentFlags,
    },

    DivideTruncate {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Exponent {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Compare {
        lhs: ExpRef,
        rhs: ExpRef,
        operation: Compare,
    },

    And {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Or {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    FunctionDeclaration {
        binds: Vec<String>,
        params: Vec<String>,
        body: ExpRef,
    },

    Return {
        value: ExpRef,
    },

    ListDeclaration {
        values: Vec<ExpRef>,
    },

    ListAccess {
        target: ExpRef,
        index: ExpRef,
    },
}

bitflags! {
    pub struct AssignmentFlags: u8 {
        const LET = 0b00000001;
        const CONDITIONAL = 0b00000010;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Compare {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}
