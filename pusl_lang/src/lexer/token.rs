use crate::backend::object::Value;
use garbage::ManagedPool;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::{cell::RefCell, fmt};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Null,
}

impl Literal {
    pub fn into_value(self, gc: &mut ManagedPool) -> Value {
        match self {
            Literal::Boolean(value) => Value::Boolean(value),
            Literal::Integer(value) => Value::Integer(value),
            Literal::Float(value) => Value::Float(value),
            Literal::String(value) => {
                let gc_ptr = gc.place_in_heap(value);
                Value::String(gc_ptr)
            }
            Literal::Null => Value::Null,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Symbol {
    OpenParenthesis,
    CloseParenthesis,
    OpenSquareBracket,
    CloseSquareBracket,
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
    And,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockType {
    If,
    Else,
    ElseIf,
    While,
    For,
    Cmp,
    Function,
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
                writeln!(f, "{:?}", tokens)
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
        writeln!(f, "{:?}", self.line)?;
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
    This,
    Self_,
    Return,
    Fn,
    Import,
    As,
    Yield,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Literal(Literal),
    Block(BlockType),
    Reference(String),
    Symbol(Symbol),
    Keyword(Keyword),
}
