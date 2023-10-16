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

use super::opcode::{ByteCodeArray, OpCode};

#[derive(Serialize, Deserialize)]
pub struct ByteCodeFile {
    pub base_func: BasicFunction,
    pub imports: Vec<Import>,
}

impl Debug for ByteCodeFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            write!(
                f,
                "ByteCode(Imports: {}, {:?})",
                self.imports.len(),
                self.base_func
            )?;
        } else {
            writeln!(f, "ByteCode")?;
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorCatch {
    pub begin: usize,
    pub filter: usize,
    pub yoink: usize,
    pub variable_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub binds: Vec<String>,
    pub literals: Vec<Literal>,
    pub references: Vec<String>,
    pub catches: Vec<ErrorCatch>,
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
                " - lits: {}, refs: {}, catches: {}, code: {}",
                self.literals.len(),
                self.references.len(),
                self.catches.len(),
                self.code.len(),
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
            writeln!(f, "Catches:")?;
            for (index, catch) in self.catches.iter().enumerate() {
                writeln!(
                    f,
                    "    {:3}; {}:{}, {} -> {}",
                    index, catch.begin, catch.filter, catch.variable_name, catch.yoink
                )?;
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

    fn new(args: Vec<String>, binds: Vec<String>) -> Function {
        Function {
            args,
            binds,
            literals: vec![],
            references: vec![],
            code: ByteCodeArray::new(),
            catches: vec![],
            is_generator: false,
        }
    }
}

pub fn linearize_file(file: ParsedFile) -> ByteCodeFile {
    let ParsedFile { expr, imports } = file;
    let func = linearize(expr, vec![], vec![]);
    let bcf = ByteCodeFile {
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
            func.function.code.push(OpCode::Modulus);
            true
        }
        Expression::Literal { value } => {
            let literal_index = func.function.add_literal(value);
            func.function.code.push(OpCode::Literal(literal_index));
            true
        }
        Expression::ThisReference => {
            func.function.code.push(OpCode::PushThis);
            true
        }
        Expression::Reference { target } => {
            let reference_index = func.function.add_reference(target);
            func.function
                .code
                .push(OpCode::PushReference(reference_index));
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
                .push(OpCode::FunctionCall(num_args));
            true
        }
        Expression::FieldAccess { target, name } => {
            linearize_exp_ref(target, func, true);
            let reference_index = func.function.add_reference(name);
            func.function.code.push(OpCode::FieldAccess(reference_index));
            true
        }
        Expression::Addition { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Addition);
            true
        }
        Expression::Subtract { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Subtraction);
            true
        }
        Expression::Negate { operand } => {
            linearize_exp_ref(operand, func, true);
            func.function.code.push(OpCode::Negate);
            true
        }
        Expression::Multiply { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Multiply);
            true
        }
        Expression::Divide { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Multiply);
            true
        }
        Expression::Elvis { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            func.function.code.push(OpCode::Duplicate);
            func.function.code.push(OpCode::IsNull);
            func.function.code.push(OpCode::Negate);
            let first_jump_setter = func.function.code.place_jump(true);
            func.function.code.push(OpCode::Pop);
            linearize_exp_ref(rhs, func, true);
            first_jump_setter(&mut func.function.code, None);
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
                    let skip_setter = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function.code.push(OpCode::Duplicate);
                        func.function.code.push(OpCode::FieldAccess(target_index));
                        func.function.code.push(OpCode::IsNull);
                        func.function.code.push(OpCode::Negate);
                        Some(func.function.code.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function.code.push(OpCode::AssignField(target_index, flags.intersects(AssignmentFlags::LET)));
                    if let Some(jump_setter) = skip_setter {
                        jump_setter(&mut func.function.code, None);
                    }
                }
                AssignAccess::Reference { name } => {
                    let target_index = func.function.add_reference(name);
                    let skip_setter = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function
                            .code
                            .push(OpCode::PushReference(target_index));
                        func.function.code.push(OpCode::IsNull);
                        func.function.code.push(OpCode::Negate);
                        Some(func.function.code.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.function
                        .code
                        .push(OpCode::AssignReference(target_index, flags.intersects(AssignmentFlags::LET)));
                    if let Some(jump_setter) = skip_setter {
                        jump_setter(&mut func.function.code, None);
                    }
                }
                AssignAccess::Array { target, index } => {
                    linearize_exp_ref(target, func, true);
                    linearize_exp_ref(index, func, true);

                    let skip_setter = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.function
                            .code
                            .push(OpCode::DuplicateDeep(1));
                        let pool_index = func.function.add_reference(String::from("@index_get"));
                        func.function.code.push(OpCode::FieldAccess(pool_index));
                        func.function
                            .code
                            .push(OpCode::DuplicateDeep(1));
                        func.function
                            .code
                            .push(OpCode::FunctionCall(1));
                        func.function.code.push(OpCode::IsNull);
                        func.function.code.push(OpCode::Negate);
                        Some(func.function.code.place_jump(true))
                    } else {
                        None
                    };
                    func.function
                        .code
                        .push(OpCode::DuplicateDeep(1));
                    let pool_index = func.function.add_reference(String::from("@index_set"));
                    func.function.code.push(OpCode::FieldAccess(pool_index));
                    func.function
                        .code
                        .push(OpCode::DuplicateDeep(1));
                    linearize_exp_ref(expression, func, true);
                    func.function
                        .code
                        .push(OpCode::FunctionCall(2));
                    if let Some(jump_setter) = skip_setter {
                        jump_setter(&mut func.function.code, None);
                    }

                    func.function.code.push(OpCode::Pop);
                    func.function.code.push(OpCode::Pop);
                }
            };

            false
        }
        Expression::DivideTruncate { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function
                .code
                .push(OpCode::DivideTruncate);
            true
        }
        Expression::Exponent { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Exponent);
            true
        }
        Expression::Compare {
            lhs,
            rhs,
            operation,
        } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Compare(operation));
            true
        }
        Expression::And { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::And);
            true
        }
        Expression::Or { lhs, rhs } => {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.function.code.push(OpCode::Or);
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
                .push(OpCode::PushFunction(index));
            true
        }
        Expression::Return { value } => {
            linearize_exp_ref(value, func, true);
            func.function.code.push(OpCode::Return);
            false
        }
        Expression::ListDeclaration { values } => {
            // TODO: Create List Object
            let pool_index = func.function.add_reference(String::from("List"));
            func.function.code.push(OpCode::PushBuiltin(pool_index));
            let num_values = values.len();
            values
                .into_iter()
                .for_each(|value| linearize_exp_ref(value, func, true));
            func.function
                .code
                .push(OpCode::FunctionCall(num_values));
            true
        }
        Expression::ListAccess { target, index } => {
            linearize_exp_ref(target, func, true);
            let pool_index = func.function.add_reference(String::from("@index_get"));
            func.function.code.push(OpCode::FieldAccess(pool_index));
            linearize_exp_ref(index, func, true);
            func.function
                .code
                .push(OpCode::FunctionCall(1));
            true
        }
        Expression::SelfReference => {
            func.function.code.push(OpCode::PushSelf);
            true
        }
        Expression::Yield { value } => {
            func.function.is_generator = true;
            linearize_exp_ref(value, func, true);
            func.function.code.push(OpCode::Yield);
            false
        }
        Expression::Yeet { value } => {
            linearize_exp_ref(value, func, true);
            func.function.code.push(OpCode::Yeet);
            false
        }
    };
    match (expand_stack, created_value) {
        (true, false) => panic!(),
        (false, true) => func.function.code.push(OpCode::Pop),
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
        Branch::TryBlock {
            try_body,
            filter_expr,
            error_variable,
            yoink_body,
        } => linearize_try(try_body, filter_expr, error_variable, yoink_body, func),
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
    assert!((greater as usize) < body.len());
    assert!((equal as usize) < body.len());
    assert!((less as usize) < body.len());
    linearize_exp_ref(lhs, func, true);
    linearize_exp_ref(rhs, func, true);
    let jump_table_setter = func.function
        .code.place_cmp_jump();
    let indexes = body
        .into_iter()
        .map(|expr| {
            let start_index = func.function.code.len();
            func.function.code.push(OpCode::ScopeUp);
            linearize_exp_ref(expr, func, false);
            func.function.code.push(OpCode::ScopeDown);
            let jump_out_setter = func.function.code.place_jump(false);
            (start_index, jump_out_setter)
        })
        .collect::<Vec<_>>();
    let jump_targets = [
        indexes[greater as usize].0 as usize,
        indexes[less as usize].0 as usize,
        indexes[equal as usize].0 as usize,
    ];
    jump_table_setter(&mut func.function.code, jump_targets);
    let jump_out_to = func.function.code.len();

    indexes.into_iter().for_each(|(_, jump_out_setter)| {
        jump_out_setter(&mut func.function.code, Some(jump_out_to));
    });
}

fn linearize_try(
    try_body: ExpRef,
    filter_expr: ExpRef,
    error_variable: String,
    yoink_body: ExpRef,
    func: &mut BasicFunction,
) {
    let try_begin_index = func.function.code.len();
    linearize_exp_ref(try_body, func, false);
    let skip_index_setter = func.function.code.place_jump(false);
    let try_filter_index = func.function.code.len();
    linearize_exp_ref(filter_expr, func, true);
    let try_yoink_index = func.function.code.len();
    linearize_exp_ref(yoink_body, func, false);
    skip_index_setter(&mut func.function.code, None);
    func.function.catches.push(ErrorCatch {
        begin: try_begin_index,
        filter: try_filter_index,
        yoink: try_yoink_index,
        variable_name: error_variable,
    });
}

fn linearize_for(variable: String, iterable: ExpRef, body: ExpRef, func: &mut BasicFunction) {
    linearize_exp_ref(iterable, func, true);
    let condition_idx = func.function.code.len();
    func.function.code.push(OpCode::Duplicate);
    let has_next_reference = func.function.add_reference("hasNext".to_string());
    func.function.code.push(OpCode::FieldAccess(has_next_reference));
    func.function
        .code
        .push(OpCode::FunctionCall(0));
    func.function.code.push(OpCode::Negate);
    let loop_end_setter = func.function.code.place_jump(true);
    func.function.code.push(OpCode::ScopeUp);
    // AssignReference, // 1 Stack Value (value) and 2 opcode (reference, type)
    func.function.code.push(OpCode::Duplicate);
    let next_reference = func.function.add_reference("next".to_string());
    func.function.code.push(OpCode::FieldAccess(next_reference));
    func.function
        .code
        .push(OpCode::FunctionCall(0));

    let target_idx = func.function.add_reference(variable);
    func.function
        .code
        .push(OpCode::AssignReference(target_idx, true));
    linearize_exp_ref(body, func, false);
    func.function.code.push(OpCode::ScopeDown);
    func.function.code.push(OpCode::jump(false, condition_idx));

    loop_end_setter(&mut func.function.code, None);
    func.function.code.push(OpCode::Pop);
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
            let jump_setter = func.function.code.place_jump(true);
            (jump_setter, body)
        })
        .collect::<Vec<_>>();
    if let Some(else_expr) = last {
        func.function.code.push(OpCode::ScopeUp);
        linearize_exp_ref(else_expr, func, false);
        func.function.code.push(OpCode::ScopeDown);
    }
    let jump_to_end_setter = func.function.code.place_jump(false);
    let place_bodies = place_conditions
        .into_iter()
        .map(|(jump_setter, body)| {
            jump_setter(&mut func.function.code, None);
            func.function.code.push(OpCode::ScopeUp);
            linearize_exp_ref(body, func, false);
            func.function.code.push(OpCode::ScopeDown);
            let jump_to_end_setter = func.function.code.place_jump(false);
            jump_to_end_setter
        })
        .collect::<Vec<_>>();
    let jump_to = func.function.code.len();
    place_bodies.into_iter().for_each(|jump_setter| {
        jump_setter(&mut func.function.code, Some(jump_to));
    });
    jump_to_end_setter(&mut func.function.code, Some(jump_to));
}

fn linearize_while(condition: ExpRef, body: ExpRef, func: &mut BasicFunction) {
    let begin_index = func.function.code.len();
    linearize_exp_ref(condition, func, true);
    func.function.code.push(OpCode::Negate);
    let condition_jump_setter = func.function.code.place_jump(true);
    func.function.code.push(OpCode::ScopeUp);
    linearize_exp_ref(body, func, false);
    func.function.code.push(OpCode::ScopeDown);
    func.function.code.push(OpCode::jump(false, begin_index));
    condition_jump_setter(&mut func.function.code, None);
}
