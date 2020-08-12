use pusl_lang::lexer::lex;

const SMALL_SOURCE: &'static str = include_str!("../resources/small_source.pusl");

#[test]
fn small_test() {
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    for root in roots {
        print!("{:?}", root);
    }
}