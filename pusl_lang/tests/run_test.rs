mod test_util;

use pusl_lang::backend::linearize::{linearize_file, ByteCodeFile};
use pusl_lang::backend::{startup, ExecContext};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use test_util::compare_test_eq;
use std::path::PathBuf;

const SECOND_SOURCE: &'static str = include_str!("../../resources/secondary_source.pusl");

fn test_resolve(path: Vec<String>) -> Option<ByteCodeFile> {
    assert_eq!(path.join("/"), "secondary_source");
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    Some(code)
}

const GENERATOR_SOURCE: &'static str = include_str!("../../resources/generator.pusl");

#[test]
fn run_generator_test() {
    let lines = GENERATOR_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let path = PathBuf::from("generator.pusl");

    let mut ctx = ExecContext::default();
    let mut output = Vec::new();
    ctx.stream = Some(&mut output);

    startup(code, path, ctx);
    let actual = String::from_utf8(output).expect("Invalid UTF8 in test output");

    compare_test_eq(&actual, "run", "generator")
}

const SIMPLE_SOURCE: &'static str = include_str!("../../resources/simple_program.pusl");

#[test]
fn run_simple_test() {
    let lines = SIMPLE_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let path = PathBuf::from("simple_program.pusl");

    let mut ctx = ExecContext::default();
    ctx.resolve = test_resolve;
    let mut output = Vec::new();
    ctx.stream = Some(&mut output);

    startup(code, path, ctx);
    let actual = String::from_utf8(output).expect("Invalid UTF8 in test output");

    compare_test_eq(&actual, "run", "small")
}

const ERROR_SOURCE: &'static str = include_str!("../../resources/errors.pusl");
#[test]
fn run_error_test() {
    let lines = ERROR_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let path = PathBuf::from("errors.pusl");

    let mut ctx = ExecContext::default();
    let mut output = Vec::new();
    ctx.stream = Some(&mut output);

    startup(code, path, ctx);
    let actual = String::from_utf8(output).expect("Invalid UTF8 in test output");

    compare_test_eq(&actual, "run", "error")
}

const FIBB_SOURCE: &'static str = include_str!("../../resources/fibb.pusl");

#[test]
fn run_fibb_test() {
    let lines = FIBB_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let path = PathBuf::from("../../resources/fibb.pusl");

    let mut ctx = ExecContext::default();
    let mut output = Vec::new();
    ctx.stream = Some(&mut output);

    startup(code, path, ctx);
    let actual = String::from_utf8(output).expect("Invalid UTF8 in test output");

    compare_test_eq(&actual, "run", "fibb")
}

const YOINK_SOURCE: &'static str = include_str!("../../resources/yoink_filter.pusl");

#[test]
fn run_yoink_test() {
    let lines = YOINK_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let path = PathBuf::from("../../resources/yoink.pusl");

    let mut ctx = ExecContext::default();
    let mut output = Vec::new();
    ctx.stream = Some(&mut output);

    startup(code, path, ctx);
    let actual = String::from_utf8(output).expect("Invalid UTF8 in test output");

    compare_test_eq(&actual, "run", "yoink")
}