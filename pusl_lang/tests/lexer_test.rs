use pusl_lang::lexer::lex;
use pusl_lang::test_util::compare_test;

const SMALL_SOURCE: &'static str = include_str!("../../resources/small_source.pusl");

#[test]
fn lex_small_test() {
    eprintln!("{}", std::env::current_dir().unwrap().to_string_lossy());
    let lines = SMALL_SOURCE.lines();
    let actual = lex(lines);

    compare_test(&actual, "lexer", "small", |expected, actual| {
        for (idx, (ex_unit, ac_unit)) in expected.into_iter().zip(actual).enumerate() {
            assert_eq!(ex_unit, ac_unit, "Lex units at idx {idx} are not equal")
        }
        assert_eq!(expected.len(), actual.len());
    });
}
