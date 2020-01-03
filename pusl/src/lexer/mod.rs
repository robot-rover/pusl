//! The lexer takes the raw source code and changes each line into a list of tokens.
//! Then, the lexer uses the indentation data and changes it into a hierarchy of tokens.
//! This hierarchy is taken in by the parser which assembles it into logical units.
//! This module finds syntactical errors.

use crate::lexer::peek_while::{peek_while, PeekWhile};
use crate::lexer::token::BlockType::*;
use crate::lexer::token::LexUnit::Statement;
use crate::lexer::token::Literal::{Boolean, Float, Integer};
use crate::lexer::token::Symbol::*;
use crate::lexer::token::{Block, BlockType, LexUnit, Literal, Symbol, Token};
use std::iter::Peekable;
use std::str::Chars;

pub mod peek_while;
pub mod token;

type Source<'a> = Peekable<Chars<'a>>;

//Todo: Support Non-Ascii

pub fn lex<'a, I>(lines: I) -> Vec<LexUnit>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut iter = lines.into_iter();
    //    iter.by_ref().map(lex_line).for_each(|e| println!("{:?}", e));
    let mut iter = iter.map(lex_line).peekable();
    let mut roots = Vec::new();
    while let Some(root) = lex_internal(&mut iter, 0) {
        roots.push(root);
    }

    roots
}

fn lex_internal<I>(stream: &mut Peekable<I>, indent: usize) -> Option<LexUnit>
where
    I: Iterator<Item = (Vec<Token>, usize)>,
{
    if let Some((tokens, indentation)) = stream.next() {
        assert_eq!(indent, indentation);
        let mut children = Vec::new();
        while stream.peek().map_or(false, |e| e.1 == indent + 1) {
            if let Some(child) = lex_internal(stream, indent + 1) {
                children.push(child)
            } else {
                panic!();
            }
        }
        if children.is_empty() {
            Some(LexUnit::Statement(tokens))
        } else {
            let first = tokens.first();
            let second = tokens.get(1);
            if let Some(&Token::Block(block_type)) = first {
                let mut return_type = block_type;
                if let BlockType::If = block_type {
                    if let Some(&Token::Block(BlockType::Else)) = second {
                        return_type = BlockType::ElseIf;
                    }
                }
                Some(LexUnit::Block(Block {
                    kind: return_type,
                    line: tokens,
                    children,
                }))
            } else {
                panic!("Block token encountered with no children")
            }
        }
    } else {
        None
    }
}

fn read_identifier(line: &mut Source) -> String {
    peek_while(line, |&c| c.is_ascii_alphanumeric() || c == '_').collect::<String>()
}

// TODO: Hex Literals
fn read_numeric_literal(line: &mut Source) -> Literal {
    let result = peek_while(line, |&c| c.is_digit(10) || c == '.').collect::<String>();
    if result.contains(".") {
        Float(result.parse().unwrap())
    } else {
        Integer(result.parse().unwrap())
    }
}

fn read_symbol(line: &mut Source) -> Symbol {
    let c = line.next().unwrap();
    match c {
        '(' => OpenParenthesis,
        ')' => CloseParenthesis,
        ',' => Comma,
        '.' => Period,
        ':' => Colon,
        ';' => SemiColon,
        '+' => Plus,
        '-' => Minus,
        '*' => {
            if line.peek().map_or(false, |&c| c == '*') {
                line.next();
                DoubleStar
            } else {
                Star
            }
        }
        '/' => {
            if line.peek().map_or(false, |&c| c == '/') {
                line.next();
                DoubleSlash
            } else {
                Slash
            }
        }
        '=' => {
            if line.peek().map_or(false, |&c| c == '=') {
                line.next();
                DoubleEquals
            } else {
                Equals
            }
        }
        '<' => {
            if line.peek().map_or(false, |&c| c == '=') {
                line.next();
                LessEquals
            } else {
                Less
            }
        }
        '>' => {
            if line.peek().map_or(false, |&c| c == '=') {
                line.next();
                GreaterEquals
            } else {
                Greater
            }
        }
        '!' => {
            if line.peek().map_or(false, |&c| c == '=') {
                line.next();
                NotEquals
            } else {
                ExclamationPoint
            }
        }
        '?' => match line.next() {
            Some(':') => Elvis,
            Some('=') => ConditionalAssignment,
            _ => panic!("Unrecognized Symbol"),
        },

        _ => panic!("Unrecognized Symbol"),
    }
}

fn read_string_literal(line: &mut Source) -> String {
    let quote = line.next().unwrap();
    assert_eq!(quote, '"');
    line.take_while(|&c| c != '"').collect::<String>()
}

fn lex_line(line: &str) -> (Vec<Token>, usize) {
    let mut cursor: Source = line.chars().peekable();
    let indentation = peek_while(&mut cursor, |&c| c == ' ').count();
    let mut tokens = Vec::new();
    while let Some(&c) = cursor.peek() {
        if c.is_ascii_alphabetic() {
            let ident = read_identifier(&mut cursor);
            let token = match ident.as_str() {
                "for" => Some(Token::Block(For)),
                "if" => Some(Token::Block(If)),
                "else" => Some(Token::Block(Else)),
                "in" => Some(Token::Symbol(In)),
                "while" => Some(Token::Block(While)),
                "compare" => Some(Token::Block(Cmp)),
                "to" => Some(Token::Symbol(To)),
                "true" => Some(Token::Literal(Boolean(true))),
                "false" => Some(Token::Literal(Boolean(false))),
                "let" => Some(Token::Let),
                _ => None,
            }
            .unwrap_or_else(|| Token::Reference(ident));
            tokens.push(token);
        } else if c.is_digit(10) {
            tokens.push(Token::Literal(read_numeric_literal(&mut cursor)));
        } else if c == '"' {
            tokens.push(Token::Literal(Literal::String(read_string_literal(
                &mut cursor,
            ))));
        } else if c == ' ' {
            peek_while(&mut cursor, |&c| c == ' ').count();
            tokens.push(Token::WhiteSpace);
        } else {
            tokens.push(Token::Symbol(read_symbol(&mut cursor)));
        }
    }

    (tokens, indentation)
}
