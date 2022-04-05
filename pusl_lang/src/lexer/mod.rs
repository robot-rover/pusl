//! The lexer takes the raw source code and changes each line into a list of tokens.
//! Then, the lexer uses the indentation data and changes it into a hierarchy of tokens.
//! This hierarchy is taken in by the parser which assembles it into logical units.
//! This module finds syntactical errors.

use crate::lexer::peek_while::peek_while;
use crate::lexer::token::Symbol::*;
use crate::lexer::token::{Block, BlockType, Keyword, LexUnit, Literal, Symbol, Token};
use std::cmp;
use std::iter::Peekable;
use std::str::Chars;
use crate::backend::linearize::OpCode::Modulus;

pub mod peek_while;
pub mod token;

type Source<'a> = Peekable<Chars<'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndentChar {
    Tab,
    Space,
}

impl IndentChar {
    fn from_char(c: char) -> Option<IndentChar> {
        match c {
            ' ' => Some(IndentChar::Space),
            '\t' => Some(IndentChar::Tab),
            _ => None,
        }
    }

    fn compare(old: &[IndentChar], new: &[IndentChar]) -> Option<cmp::Ordering> {
        for (old_char, new_char) in old.iter().zip(new.iter()) {
            if old_char != new_char {
                return None;
            }
        }
        Some(old.len().cmp(&new.len()))
    }
}

//Todo: Support Non-Ascii

pub fn lex<'a, I>(lines: I) -> Vec<LexUnit>
where
    I: IntoIterator<Item = &'a str>,
{
    let iter = lines.into_iter();
    let indent_iter = iter.map(lex_line).filter(|(line, _)| !line.is_empty());
    let mut last_indent = Vec::new();
    let mut lines = Vec::new();
    for (tokens, indent_tokens) in indent_iter {
        assert!(IndentChar::compare(&last_indent, &indent_tokens).is_some());
        lines.push((tokens, indent_tokens.len()));
        last_indent = indent_tokens;
    }

    let mut roots = Vec::new();
    let mut iter = lines.into_iter().peekable();

    while let Some(root) = lex_internal(&mut iter) {
        roots.push(root);
    }

    roots
}

fn lex_internal<I>(stream: &mut Peekable<I>) -> Option<LexUnit>
where
    I: Iterator<Item = (Vec<Token>, usize)>,
{
    if let Some((tokens, indentation)) = stream.next() {
        let mut children = Vec::new();
        let mut children_indent: Option<usize> = None;
        while stream
            .peek()
            .map_or(false, |(_, child_indent)| indentation < *child_indent)
        {
            if let Some(children_indent) = children_indent {
                let (_, this_indent) = stream.peek().unwrap();
                assert_eq!(children_indent, *this_indent);
            } else {
                let (_, this_indent) = stream.peek().unwrap();
                children_indent = Some(*this_indent);
            }
            if let Some(child) = lex_internal(stream) {
                children.push(child)
            } else {
                panic!();
            }
        }

        if !children.is_empty() {
            assert_eq!(tokens.last(), Some(&Token::Symbol(Symbol::Colon)));
            let first = tokens.first();
            let second = tokens.get(1);
            if let Some(&Token::Block(block_type)) = first {
                let mut return_type = block_type;
                if let BlockType::Else = block_type {
                    if let Some(&Token::Block(BlockType::If)) = second {
                        return_type = BlockType::ElseIf;
                    }
                }
                Some(LexUnit::Block(Block {
                    kind: return_type,
                    line: tokens,
                    children,
                }))
            } else if tokens.contains(&Token::Keyword(Keyword::Fn)) {
                Some(LexUnit::Block(Block {
                    kind: BlockType::Function,
                    line: tokens,
                    children,
                }))
            } else {
                panic!("Unrecognized Block Type");
            }
        } else {
            Some(LexUnit::Statement(tokens))
        }
    } else {
        None
    }
}

fn read_identifier(line: &mut Source) -> String {
    peek_while(line, |&c| c.is_ascii_alphanumeric() || c == '_' || c == '@').collect::<String>()
}

// TODO: Hex Literals
fn read_numeric_literal(line: &mut Source) -> Literal {
    let result = peek_while(line, |&c| c.is_digit(10) || c == '.').collect::<String>();
    if result.contains('.') {
        Literal::Float(result.parse().unwrap())
    } else {
        Literal::Integer(result.parse().unwrap())
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
        '[' => OpenSquareBracket,
        ']' => CloseSquareBracket,
        '&' => And,
        '|' => Or,
        '%' => Percent,
        _ => panic!("Unrecognized Symbol"),
    }
}

fn read_string_literal(line: &mut Source) -> String {
    let quote = line.next().unwrap();
    assert_eq!(quote, '"');
    let mut string = String::new();
    while let Some(c) = line.next() {
        if c == '"' {
            break;
        } else if c == '\\' {
            let escaped = match line.next().expect("expected character after backslash") {
                'n' => '\n',
                't' => '\t',
                _ => panic!("Illegal Character after backslash"),
            };
            string.push(escaped);
        } else {
            string.push(c);
        }
    }
    string
}

fn lex_line(line: &str) -> (Vec<Token>, Vec<IndentChar>) {
    let mut cursor: Source = line.chars().peekable();
    let indentation = peek_while(cursor.by_ref(), |&c| c == ' ' || c == '\t')
        .map(|c| IndentChar::from_char(c).unwrap())
        .collect();
    let mut tokens = Vec::new();
    while let Some(&c) = cursor.peek() {
        if c.is_ascii_alphabetic() || c == '@' {
            let ident = read_identifier(&mut cursor);
            let token = match ident.as_str() {
                "for" => Some(Token::Block(BlockType::For)),
                "if" => Some(Token::Block(BlockType::If)),
                "else" => Some(Token::Block(BlockType::Else)),
                "in" => Some(Token::Keyword(Keyword::In)),
                "while" => Some(Token::Block(BlockType::While)),
                "compare" => Some(Token::Block(BlockType::Cmp)),
                "to" => Some(Token::Keyword(Keyword::To)),
                "true" => Some(Token::Literal(Literal::Boolean(true))),
                "false" => Some(Token::Literal(Literal::Boolean(false))),
                "let" => Some(Token::Keyword(Keyword::Let)),
                "this" => Some(Token::Keyword(Keyword::This)),
                "self" => Some(Token::Keyword(Keyword::Self_)),
                "return" => Some(Token::Keyword(Keyword::Return)),
                "null" => Some(Token::Literal(Literal::Null)),
                "fn" => Some(Token::Keyword(Keyword::Fn)),
                "import" => Some(Token::Keyword(Keyword::Import)),
                "as" => Some(Token::Keyword(Keyword::As)),
                "yield" => Some(Token::Keyword(Keyword::Yield)),
                "yeet" => Some(Token::Keyword(Keyword::Yeet)),
                "try" => Some(Token::Block(BlockType::Try)),
                "yoink" => Some(Token::Block(BlockType::Yoink)),
                _ => None,
            }
            .unwrap_or(Token::Reference(ident));
            tokens.push(token);
        } else if c.is_digit(10) {
            tokens.push(Token::Literal(read_numeric_literal(&mut cursor)));
        } else if c == '"' {
            tokens.push(Token::Literal(Literal::String(read_string_literal(
                &mut cursor,
            ))));
        } else if c == ' ' {
            peek_while(&mut cursor, |&c| c == ' ').count();
        } else {
            tokens.push(Token::Symbol(read_symbol(&mut cursor)));
        }
    }

    (tokens, indentation)
}
