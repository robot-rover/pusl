mod test_util;

use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use test_util::compare_test_eq;

const SMALL_SOURCE: &'static str = include_str!("../../resources/small_source.pusl");

#[test]
fn parse_small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);

    compare_test_eq(&ast.expr, "parse", "small");
}

#[test]
fn error_test() {
    let lines = include_str!("../../resources/errors.pusl").lines();
    let roots = lex(lines);
    let ast = parse(roots);

    compare_test_eq(&ast.expr, "parse", "error");
}
