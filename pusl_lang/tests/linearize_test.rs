mod test_util;

use pusl_lang::backend::{
    linearize::{linearize_file, BasicFunction, ByteCodeFile, Function},
    opcode::ByteCodeArray,
};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use test_util::compare_test;

const SMALL_SOURCE: &'static str = include_str!("../../resources/simple_program.pusl");

const SECOND_SOURCE: &'static str = include_str!("../../resources/secondary_source.pusl");

fn check_code_equal(expect: &ByteCodeArray, actual: &ByteCodeArray, context: &str) {
    let expect_int = expect.iter().collect::<Vec<_>>();
    let actual_int = actual.iter().collect::<Vec<_>>();

    assert_eq!(expect_int, actual_int, "Function {} code mismatch", context);
}

fn check_function_equal(expect: &Function, actual: &Function, context: &str) {
    let Function {
        args: ex_args,
        binds: ex_binds,
        literals: ex_literals,
        references: ex_references,
        code: ex_code,
        catches: ex_catches,
        is_generator: ex_is_generator,
    } = expect;
    let Function {
        args: ac_args,
        binds: ac_binds,
        literals: ac_literals,
        references: ac_references,
        code: ac_code,
        catches: ac_catches,
        is_generator: ac_is_generator,
    } = actual;

    assert_eq!(ex_args, ac_args, "Function {} arguments mismatch", context);
    assert_eq!(ex_binds, ac_binds, "Function {} binds mismatch", context);
    assert_eq!(
        ex_literals, ac_literals,
        "Function {} literals mismatch",
        context
    );
    assert_eq!(
        ex_references, ac_references,
        "Function {} references mismatch",
        context
    );
    check_code_equal(ex_code, ac_code, context);
    assert_eq!(
        ex_catches, ac_catches,
        "Function {} catches mismatch",
        context
    );
    assert_eq!(
        ex_is_generator, ac_is_generator,
        "Function {} is_generator mismatch",
        context
    );
}

fn check_basic_function_equal(expect: &BasicFunction, actual: &BasicFunction, context: &str) {
    let ex_function = expect.as_ref();
    let BasicFunction {
        sub_functions: ex_sub_functions,
        ..
    } = expect;
    let ac_function = actual.as_ref();
    let BasicFunction {
        sub_functions: ac_sub_functions,
        ..
    } = actual;

    check_function_equal(ex_function, ac_function, context);

    for (idx, (esf, asf)) in ex_sub_functions
        .into_iter()
        .zip(ac_sub_functions)
        .enumerate()
    {
        check_basic_function_equal(esf, asf, format!("{}/{}", context, idx).as_str());
    }
}

fn check_bcf_equal(expect: &ByteCodeFile, actual: &ByteCodeFile) {
    let ByteCodeFile {
        base_func: ex_base_func,
        imports: ex_imports,
    } = expect;
    let ByteCodeFile {
        base_func: ac_base_func,
        imports: ac_imports,
    } = actual;

    for (idx, (ex_import, ac_import)) in ex_imports.into_iter().zip(ac_imports).enumerate() {
        assert_eq!(ex_import, ac_import, "Import #{} doesn't match", idx)
    }

    assert_eq!(
        ex_imports.len(),
        ac_imports.len(),
        "Imports length mismatch"
    );

    check_basic_function_equal(ex_base_func, ac_base_func, "root");
}

#[test]
fn error_test() {
    let lines = include_str!("../../resources/errors.pusl").lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);

    compare_test(&code, "linear", "error", check_bcf_equal);
}

#[test]
fn linear_simple_test() {
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);

    compare_test(&code, "linear", "simple", check_bcf_equal);
}

#[test]
fn linear_small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);

    compare_test(&code, "linear", "small", check_bcf_equal);
}
