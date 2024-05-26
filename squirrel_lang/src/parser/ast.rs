type Ident = String;

pub enum Literal {
    Integer(i64),
    Number(f64),
    String(String),
    Null,
}

pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Greater,
    Less,
    Compare,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    AShr,
    Comma,
}

enum UnaryOp {
    Neg,
    Not,
    BitNot,
    TypeOf,
    Incr,
    Decr,
    Clone,
}

pub struct Function {
    args: Vec<Ident>,
    body: StateRef,
}

pub type StateRef = Box<Statement>;
pub enum Statement {
    Block(Vec<Statement>),
    Expr(Expr),
    IfElse(Expr, StateRef, StateRef),
    While(Expr, StateRef),
    DoWhile(Expr, StateRef),
    Switch(Expr, Vec<(Expr, Statement)>),
    For {
        // TODO: Should be localdec
        init: StateRef,
        cond: Expr,
        incr: Expr,
        body: StateRef,
    },
    Foreach {
        index_id: Ident,
        value_id: Ident,
        iterable: Expr,
        body: StateRef,
    },
    Break,
    Continue,
    Return(Expr),
    Yield(Expr),
    LocalDec(Vec<(Ident, Option<Expr>)>),
    TryCatch(StateRef, Ident, StateRef),
    Throw(Expr),
    Const(Ident, Literal),
    // TODO Enum
    Empty,
}

pub type ExprRef = Box<Expr>;
pub enum Expr {
    Literal(Literal),
    TableDecl(Vec<(Expr, Expr)>),
    ArrayDecl(Vec<Expr>),
    FunctionDef(Function),
    ClassDef {
        constructor: Function,
        members: Vec<(Expr, Expr)>,
    },
    Assign(Ident, ExprRef),
    NewSlot(Ident, ExprRef),
    Ternary {
        cond: ExprRef,
        true_expr: ExprRef,
        false_expr: ExprRef,
    },
    BinaryOp {
        op: BinaryOp,
        lhs: ExprRef,
        rhs: ExprRef,
    },
    UnaryOp(UnaryOp, ExprRef),
}
