use std::borrow::Cow;

use logos::{Lexer, Logos, Span};

#[derive(Debug, PartialEq, Logos)]
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
    #[token(".")]
    Period,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    Semicolon,
    #[token("\n")]
    Newline,
    #[token("@")]
    AtSymbol,
    // Literals
    #[regex(r#""([^"]|(\\")|\n)*""#, |lex| escape_str(trim_str(lex.slice(), 1, 1)))]
    #[regex(r#"@"([^"]|\n)*""#, |lex| trim_str(lex.slice(), 2, 1).to_string())]
    String(String),
    Integer(i64),
    Number(f64),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Boolean(bool),
    #[token("null")]
    Null,
    // Comments
    Comment(String),
}

#[derive(Debug, PartialEq, Logos)]
pub enum EscapedString<'s> {
    #[regex(r"[^\\]+")]
    Verbatim(&'s str),
    #[token(r"\n", |_| "\n".to_string())]
    #[token(r#"\""#, |_| "\"".to_string())]
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

fn escape_str(s: &str) -> Result<String, ()> {
    EscapedString::lexer(s)
        .map(|res| res.map(|part| part.into()))
        .collect::<Result<Vec<Cow<str>>, _>>()
        .map(|v| v.join(""))
}
