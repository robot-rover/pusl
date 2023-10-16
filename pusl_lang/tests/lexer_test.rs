mod test_util;

use pusl_lang::lexer::{lex, token::LexUnit};
use test_util::compare_test;

const SMALL_SOURCE: &'static str = include_str!("../../resources/small_source.pusl");

fn compare_lex_unit(expected: &Vec<LexUnit>, actual: &Vec<LexUnit>) {
    for (idx, (ex_unit, ac_unit)) in expected.into_iter().zip(actual).enumerate() {
        assert_eq!(ex_unit, ac_unit, "Lex units at idx {idx} are not equal")
    }
    assert_eq!(expected.len(), actual.len());
}

#[test]
fn lex_small_test() {
    let lines = SMALL_SOURCE.lines();
    let actual = lex(lines);

    compare_test(&actual, "lexer", "small", compare_lex_unit);
}

#[test]
fn error_test() {
    let lines = include_str!("../../resources/errors.pusl").lines();
    let actual = lex(lines);

    compare_test(&actual, "lexer", "error", compare_lex_unit);
}
