use std::fmt;
use std::fmt::{write, Debug, Formatter};
use bitflags::_core::fmt::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Null
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Symbol {
    OpenParenthesis,
    CloseParenthesis,
    Percent,
    Comma,
    ExclamationPoint,
    Period,
    Colon,
    SemiColon,
    Elvis,
    ConditionalAssignment,
    Plus,
    Minus,
    Star,
    DoubleStar,
    Slash,
    DoubleSlash,
    Equals,
    DoubleEquals,
    NotEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    Or,
    And
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockType {
    If,
    Else,
    ElseIf,
    While,
    For,
    Cmp,
}

pub enum LexUnit {
    Statement(Vec<Token>),
    Block(Block),
}

impl Debug for LexUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indent(f, 0)
    }
}

pub struct Block {
    pub kind: BlockType,
    pub line: Vec<Token>,
    pub children: Vec<LexUnit>,
}

impl LexUnit {
    pub fn get_tokens(&self) -> &Vec<Token> {
        match self {
            LexUnit::Statement(tokens) => tokens,
            LexUnit::Block(block) => block.get_tokens(),
        }
    }

    pub fn fmt_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            LexUnit::Statement(tokens) => {
                for _ in 0..indent {
                    write!(f, "\t")?;
                }
                write!(f, "{:?}\n", tokens)
            }
            LexUnit::Block(block) => block.fmt_indent(f, indent),
        }
    }
}

impl Block {
    pub fn fmt_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
        for _ in 0..indent {
            write!(f, "\t")?;
        }
        write!(f, "{:?}\n", self.line)?;
        for child in &self.children {
            child.fmt_indent(f, indent + 1)?
        }
        Ok(())
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.line
    }

    pub fn get_children(&self) -> &Vec<LexUnit> {
        &self.children
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indent(f, 0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Keyword {
    Let,
    In,
    To,
    This
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Literal(Literal),
    Block(BlockType),
    Reference(String),
    Symbol(Symbol),
    Keyword(Keyword),
}
