use crate::lexer::token::Literal;
use crate::parser::ExpRef;
use generational_arena::Index;

/// Syntax Blocks which are linear
/// i.e. they will never branch
pub enum Expression {
    Nullify {
        expr: ExpRef
    },

    Modulus {
        lhs: ExpRef,
        rhs: ExpRef,
    },

    Literal {
        value: Literal,
    },

    Reference {
        target: String,
    },

    Joiner {
        expressions: Vec<ExpRef>,
    },

    FunctionCall {
        target: ExpRef,
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
        target: ExpRef,
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
    }
}

bitflags! {
    pub struct AssignmentFlags: u8 {
        const LET = 0b00000001;
        const CONDITIONAL = 0b00000010;
    }
}

#[derive(Copy, Clone)]
pub enum Compare {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}
