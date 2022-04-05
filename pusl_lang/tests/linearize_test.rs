extern crate pusl_lang;

use pusl_lang::backend::linearize::linearize_file;
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;

const SMALL_SOURCE: &'static str = include_str!("../resources/simple_program.pusl");

const SECOND_SOURCE: &'static str = include_str!("../resources/secondary_source.pusl");

#[test]
fn error_test() {
    let lines = include_str!("../resources/errors.pusl").lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    println!("{:#?}", code);
}

#[test]
fn small_test() {
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    println!("{:#?}", code);

    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    println!("{:#?}", code);
}
