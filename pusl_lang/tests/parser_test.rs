extern crate pusl_lang;

use pusl_lang::lexer::lex;
use pusl_lang::parser::expression::Expression;
use pusl_lang::parser::{parse, Eval, ParsedFile};

const SMALL_SOURCE: &'static str = include_str!("../resources/small_source.pusl");

#[test]
fn small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    println!("{:?}", roots.last());
    let ast = parse(roots);
    let ParsedFile { expr, imports } = ast;
    if let Eval::Expression(Expression::Joiner { expressions }) = *expr {
        for expr in expressions {
            println!("{:#?}", expr);
        }
    }
    println!("{:#?}", imports);
}

#[test]
fn error_test() {
    let lines = include_str!("../resources/errors.pusl").lines();
    let roots = lex(lines);
    println!("{:?}", roots.last());
    let ast = parse(roots);
    let ParsedFile { expr, imports } = ast;
    if let Eval::Expression(Expression::Joiner { expressions }) = *expr {
        for expr in expressions {
            println!("{:#?}", expr);
        }
    }
    println!("{:#?}", imports);
}
