use crate::parser::ExpRef;

#[derive(Debug)]
pub struct ConditionBody {
    pub condition: ExpRef,
    pub body: ExpRef,
}

/// Syntax Blocks which branch execution flow
#[derive(Debug)]
pub enum Branch {
    TryBlock {
        try_body: ExpRef,
        filter_expr: ExpRef,
        error_variable: String,
        yoink_body: ExpRef,
    },

    IfElseBlock {
        conditions: Vec<ConditionBody>,
        last: Option<ExpRef>,
    },

    WhileLoop {
        condition: ExpRef,
        body: ExpRef,
    },

    ForLoop {
        variable: String,
        iterable: ExpRef,
        body: ExpRef,
    },

    CompareBlock {
        lhs: ExpRef,
        rhs: ExpRef,
        greater: u8,
        equal: u8,
        less: u8,
        body: Vec<ExpRef>,
    },

    Joiner {
        expressions: Vec<ExpRef>,
    },
}
