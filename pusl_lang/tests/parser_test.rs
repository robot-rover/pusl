extern crate pusl_lang;

use pusl_lang::lexer::lex;
use pusl_lang::parser::expression::Expression;
use pusl_lang::parser::{parse, Eval, ParsedFile};
use pusl_lang::test_util::compare_test_eq;

const SMALL_SOURCE: &'static str = include_str!("../../resources/small_source.pusl");

#[test]
fn parse_small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    println!("{:?}", roots.last());
    let ast = parse(roots);
    let ParsedFile { expr, imports } = &ast;
    println!("{:#?}", imports);
    if let Eval::Expression(Expression::Joiner { expressions }) = expr.as_ref() {
        for expr in expressions {
            println!("{:#?}", expr);
        }
    }

    compare_test_eq(&ast, "parse", "small");
}
