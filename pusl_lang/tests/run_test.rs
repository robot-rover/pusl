extern crate pusl_lang;

use pusl_lang::backend::linearize::{linearize_file, ByteCodeFile};
use pusl_lang::backend::{startup, ExecContext};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use std::path::PathBuf;

const SMALL_SOURCE: &'static str = include_str!("../resources/simple_program.pusl");
const SECOND_SOURCE: &'static str = include_str!("../resources/secondary_source.pusl");

fn test_resolve(path: PathBuf) -> Option<ByteCodeFile> {
    assert_eq!(path.to_str().unwrap(), "secondary_source");
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast, PathBuf::from("secondary_source"));
    Some(code)
}

#[test]
fn small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast, PathBuf::from("../resources/simple_program.pusl"));
    let ctx = ExecContext {
        resolve: test_resolve,
    };
    startup(code, ctx);
}

const FIBB_SOURCE: &'static str = include_str!("../resources/fibb.pusl");

#[test]
fn fibb_test() {
    let lines = FIBB_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast, PathBuf::from("../resources/fibb.pusl"));
    let ctx = ExecContext { resolve: |_| None };
    startup(code, ctx);
}
