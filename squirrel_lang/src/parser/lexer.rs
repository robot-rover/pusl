use std::{borrow::Cow, num::ParseIntError};
use std::str::FromStr;

use logos::{Lexer, Logos, Span};

use super::context::{LexerContext, LexerError, LexerResult};

#[derive(Debug, PartialEq, Logos)]
#[logos(extras = LexerContext)]
#[logos(error = LexerError)]
#[logos(skip r"[ \t\r\f]+")]
pub enum Token {
    // Keywords
    #[token("base")]
    Base,
    #[token("break")]
    Break,
    #[token("case")]
    Case,
    #[token("catch")]
    Catch,
    #[token("class")]
    Class,
    #[token("clone")]
    Clone,
    #[token("continue")]
    Continue,
    #[token("const")]
    Const,
    #[token("default")]
    Default,
    #[token("delete")]
    Delete,
    #[token("else")]
    Else,
    #[token("enum")]
    Enum,
    #[token("extends")]
    Extends,
    #[token("for")]
    For,
    #[token("foreach")]
    ForEach,
    #[token("function")]
    Function,
    #[token("if")]
    If,
    #[token("in")]
    In,
    #[token("local")]
    Local,
    #[token("resume")]
    Resume,
    #[token("return")]
    Return,
    #[token("switch")]
    Switch,
    #[token("this")]
    This,
    #[token("throw")]
    Throw,
    #[token("try")]
    Try,
    #[token("typeof")]
    Typeof,
    #[token("while")]
    While,
    #[token("yield")]
    Yield,
    #[token("constructor")]
    Constructor,
    #[token("instanceof")]
    InstaceOf,
    #[token("static")]
    Static,
    #[token("__LINE__")]
    LineDunder,
    #[token("__FILE__")]
    FileDunder,
    #[token("rawcall")]
    RawCall,
    // Operators
    #[token("!")]
    Not,
    #[token("!=")]
    DoesNotEqual,
    #[token("||")]
    Or,
    #[token("==")]
    Equals,
    #[token("&&")]
    And,
    #[token(">=")]
    GreaterThanEquals,
    #[token("<=")]
    LessThanEquals,
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
    #[token("<=>")]
    Compare,
    #[token("+")]
    Plus,
    #[token("+=")]
    PlusAssign,
    #[token("-")]
    Minus,
    #[token("-=")]
    MinusAssign,
    #[token("/")]
    Divide,
    #[token("/=")]
    DivideAssign,
    #[token("*")]
    Multiply,
    #[token("*=")]
    MultiplyAssign,
    #[token("%")]
    Modulus,
    #[token("%=")]
    ModulusAssign,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,
    #[token("<-")]
    NewSlot,
    #[token("=")]
    Assign,
    #[token("&")]
    BitAnd,
    #[token("^")]
    BitXor,
    #[token("|")]
    BitOr,
    #[token("~")]
    BitNot,
    #[token(">>")]
    RightShift,
    #[token("<<")]
    LeftShift,
    #[token(">>>")]
    RightShiftArith,
    // Misc Symbols
    #[token("{")]
    LeftCurlyBrace,
    #[token("}")]
    RightCurlyBrace,
    #[token("[")]
    LeftSquareBracket,
    #[token("]")]
    RightSquareBracket,
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token(".")]
    Period,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    Semicolon,
    #[token("?")]
    QuestionMark,
    #[token("\n", |lex| {
        let span = lex.span();
        debug_assert_eq!(span.start + 1, span.end, "Newline token should be a single character");
        lex.extras.log_newline(span.start);
    })]
    Newline,
    #[token("@")]
    AtSymbol,
    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    // Literals
    // TODO: Parse strings with seperate lexers
    // Todo: error if literal \n in non-verbatim string
    #[regex(r#""([^"]|(\\")|\n)*""#, |lex| escape_str(trim_str(find_newlines(lex), 1, 1)))]
    #[regex(r#"@"([^"]|\n)*""#, |lex| trim_str(find_newlines(lex), 2, 1).to_string())]
    String(String),
    // TODO: Error Handling for invalid digits
    #[regex(r"\d+", |lex| i64::from_str(lex.slice()))]
    #[regex(r"0\d+", |lex| i64::from_str_radix(&lex.slice()[1..], 8))]
    #[regex(r"0x[0-9a-fA-F]+", |lex| i64::from_str_radix(&lex.slice()[2..], 16))]
    // Todo: handle too long
    #[regex(r"'([^'])*'", |lex| lex.slice().chars().nth(1).map(|c| c as i64))]
    Integer(i64),
    // Todo: handle missing number after decimal point
    #[regex(r"\d+\.\d*", |lex| f64::from_str(lex.slice()))]
    #[regex(r"\d+(\.\d*)?e[+-]?\d*", |lex| parse_sci(lex))]
    Number(f64),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Boolean(bool),
    #[token("null")]
    Null,
    // Comments
    #[regex(r"//[^\n]*", |lex| lex.slice()[2..].to_string())]
    #[regex(r"#[^\n]*", |lex| lex.slice()[1..].to_string())]
    #[regex(r"/\*([^*]|\n|\*[^/])*\*/", |lex| trim_str(find_newlines(lex), 2, 2).to_string())]
    Comment(String),
}

#[derive(Debug, PartialEq, Logos)]
pub enum EscapedString<'s> {
    #[regex(r"[^\\]+")]
    Verbatim(&'s str),
    #[regex(r"\\.", |lex| escape_lookup(lex.slice()))]
    Escaped(String),
}

impl<'s> Into<Cow<'s, str>> for EscapedString<'s> {
    fn into(self) -> Cow<'s, str> {
        match self {
            EscapedString::Verbatim(slice) => slice.into(),
            EscapedString::Escaped(string) => string.into(),
        }
    }
}

fn trim_str(s: &str, trim_start: usize, trim_end: usize) -> &str {
    &s[trim_start..s.len() - trim_end]
}

fn escape_lookup(s: &str) -> Option<String> {
    match s {
        r"\n" => Some("\n".to_string()),
        r"\r" => Some("\r".to_string()),
        r"\t" => Some("\t".to_string()),
        r"\\" => Some("\\".to_string()),
        r#"\""# => Some("\"".to_string()),
        _ => None,
    }
}

fn escape_str<'s>(source: &'s str) -> LexerResult<String> {
    let mut escape_lexer = EscapedString::lexer(source);
    let mut fragments: Vec<Cow<_>> = Vec::new();
    while let Some(token) = escape_lexer.next() {
        match token {
            Ok(fragment) => fragments.push(fragment.into()),
            Err(_) => return Err(LexerError::General(format!("Illegal escape sequence in string: \"{}\"", escape_lexer.slice()))),
        }
    }
    Ok(fragments.join(""))
}

// TODO: This would be faster if it is rolled into the escaping logic for strings
fn find_newlines<'s>(lexer: &mut Lexer<'s, Token>) -> &'s str {
    let slice = lexer.slice();
    let span_begin = lexer.span().start;
    slice.char_indices().filter(|&(_, c)| c == '\n').for_each(|(idx, _)| lexer.extras.log_newline(span_begin + idx));
    slice
}

fn parse_sci<'s>(lexer: &Lexer<'s, Token>) -> LexerResult<f64> {
    // TODO: Error handling
    let s = lexer.slice();
    let e_loc = s.find('e').unwrap();
    let (base, exp) = s.split_at(e_loc);
    let base = f64::from_str(base)?;
    // Skip the 'e'
    let exp = i32::from_str(&exp[1..])?;
    Ok(base * 10f64.powi(exp))
}

pub fn lex(input: &str) -> Lexer<Token> {
    Token::lexer(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn single_token_test() {
        println!("Checking Basic Tokens");
        let result = lex("base").collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Base)], result);

        let result = lex("true").collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Boolean(true))], result);

        // Strings
        println!("Checking Strings");
        let result = lex(r#""Hello World""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello World".into()))], result);

        println!("Checking String Escapes");
        let result = lex(r#""Hello\"World""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String(r#"Hello"World"#.into()))], result);

        let result = lex(r#""Hello\nWorld""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\nWorld".into()))], result);

        // TODO: This should error when its not a verbatim string
        let result = lex("\"Hello\nWorld\"")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\nWorld".into()))], result);

        println!("Checking Verbatim Strings");
        let result = lex("@\"Hello\\nWorld\nNewline\"")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\\nWorld\nNewline".into()))], result);

        println!("Checking Comments");
        let result = lex("// This is a comment")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a comment".into()))], result);

        let result = lex("# This is a comment")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a comment".into()))], result);

        let result = lex("  /* This is a\n\n comment */  ")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a\n\n comment ".into()))], result);

        println!("Checkint Integer Literals");
        let result = lex("123")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Integer(123))], result);

        let result = lex("0123")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Integer(1*8*8 + 2*8 + 3))], result);

        let result = lex("0x123")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Integer(0x123)]), result);

        let result = lex("'a'")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Integer(97)]), result);

        println!("Checking Float Literals");
        let result = lex("7.")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Number(7.)]), result);

        let result = lex("4.0")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Number(4.)]), result);

        let result = lex("4.e2")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Number(4e2)]), result);

        let result = lex("4.e-2")
            .collect::<LexerResult<Vec<_>>>();
        assert_eq!(Ok(vec![Token::Number(4e-2)]), result);
    }

    #[test]
    fn squirrel_sample_test() {
        for file in fs::read_dir("../squirrel/samples/").expect("Unable to find squirrel samples directory") {
            let file = file.expect("Unable to read squirrel samples directory");
            let path = file.path();
            let contents = fs::read_to_string(&path).unwrap();
            let mut lexer = lex(&contents);
            let tokens = lexer.by_ref().collect::<LexerResult<Vec<_>>>();
            match tokens {
                Ok(tokens) => {},
                Err(e) => panic!("{:#}", e.with_context(&lexer, path.to_string_lossy().to_string())),
            }

            // TODO: Do something with tokens
        }
    }
}
