use crate::parser::{ExpRef, Expression};
pub struct ConditionBody {
    pub condition: ExpRef,
    pub body: ExpRef,
}

/// Syntax Blocks which branch execution flow
pub enum Branch {
    IfElseBlock {
        conditions: Vec<ConditionBody>,
        last: Option<ExpRef>,
    },

    WhileLoop {
        condition: ExpRef,
        body: ExpRef,
    },

    ForLoop {
        iterable: ExpRef,
        body: ExpRef,
    },

    CompareBlock {
        lhs: ExpRef,
        rhs: ExpRef,
        greater: ExpRef,
        equal: ExpRef,
        less: ExpRef,
    },
}
