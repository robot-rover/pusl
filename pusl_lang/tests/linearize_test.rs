extern crate pusl_lang;

use pusl_lang::backend::linearize::{linearize_file, ByteCodeFile, Function};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use std::path::PathBuf;

const SMALL_SOURCE_PATH: &'static str = "../resources/simple_program.pusl";
const SMALL_SOURCE: &'static str = include_str!("../resources/simple_program.pusl");

const SECOND_SOURCE: &'static str = include_str!("../resources/secondary_source.pusl");

#[test]
fn small_test() {
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast, PathBuf::from("secondary_source"));
    println!("{:#?}", code);

    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast, PathBuf::from(SMALL_SOURCE_PATH));
    println!("{:#?}", code);
}
