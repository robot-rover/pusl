//! The parser takes the token hierarchy produced by the lexer and creates an abstract syntax tree.
//! This is where grammatical errors are caught (lexer catches syntax errors).
//! This data is taken in by the linearization engine before being executed.

use crate::lexer::peek_while::peek_while;
use crate::lexer::token::Symbol::ConditionalAssignment;
use crate::lexer::token::{Block, BlockType, LexUnit, Literal, Symbol, Token};
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::AssignmentFlags;
use crate::parser::expression::Expression;
use crate::parser::expression::Compare;
use crate::parser::expression::Expression::{Assigment, FieldAccess, Reference, Subtract, Addition, Nullify, Negate};
use generational_arena::{Arena, Index};
use std::io::Read;

pub mod branch;
pub mod expression;

/// All structures in the Abstract Syntax Tree
/// Branch and Expression or different because Branch will affect execution flow
/// Expression executes in a deterministic order
enum Eval {
    Expression(Expression),
    Branch(Branch),
}

type ExpRef = Index;

fn parse<I>(source: I) -> Eval
where
    I: IntoIterator<Item = LexUnit>,
{
    let arena: Arena<Eval> = Arena::new();
    source.into_iter();

    unimplemented!()
}

fn parse_lex_unit<I>(unit_stream: &mut I, arena: &mut Arena<Eval>) -> ExpRef
where
    I: Iterator<Item = LexUnit>,
{
    let eval = match unit_stream.next() {
        Some(LexUnit::Block(block)) => Eval::Branch(parse_branch(block, unit_stream, arena)),
        Some(LexUnit::Statement(tokens)) => Eval::Expression(parse_statement(tokens, arena)),
        None => panic!(),
    };
    arena.insert(eval)
}

/// Parse a while loop
fn parse_while(block: Block, arena: &mut Arena<Eval>) -> Branch {
    assert_eq!(
        BlockType::While,
        block.kind,
        "If the function is called, should be parsing a while loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(Some(Token::Block(BlockType::While)), it.next());
        parse_to(
            it.collect::<Vec<_>>().as_mut_slice(),
            ParseTarget::Colon,
            arena,
        )
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func, arena);
    let condition = arena.insert(Eval::Expression(condition));
    Branch::WhileLoop { condition, body }
}

/// Parse a group of if, else if, ..., else blocks
fn parse_if_else<I>(if_block: Block, block_stream: &mut I, arena: &mut Arena<Eval>) -> Branch
where
    I: Iterator<Item = LexUnit>,
{
    let mut block_stream = block_stream.peekable();
    let mut conditions = Vec::<ConditionBody>::new();

    let mut if_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(
            Some(Token::Block(BlockType::If)),
            it.next(),
            "If the function is called, should be parsing an if block"
        );
        let eval = Eval::Expression(parse_to(
            it.collect::<Vec<_>>().as_mut_slice(),
            ParseTarget::Colon,
            arena,
        ));
        arena.insert(eval)
    };
    let (if_condition, if_body) = parse_condition_body(if_block, &mut if_func, arena);
    conditions.push(ConditionBody {
        condition: if_condition,
        body: if_body,
    });

    let mut elif_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(
            Some(Token::Block(BlockType::Else)),
            it.next(),
            "If the function is called, should be parsing an if else block"
        );
        assert_eq!(
            Some(Token::Block(BlockType::If)),
            it.next(),
            "If the function is called, should be parsing an if else block"
        );
        let eval = Eval::Expression(parse_to(
            it.collect::<Vec<_>>().as_mut_slice(),
            ParseTarget::Colon,
            arena,
        ));
        arena.insert(eval)
    };
    while block_stream.peek().map_or(false, |lex_unit| {
        if let LexUnit::Block(block) = lex_unit {
            block.kind == BlockType::ElseIf
        } else {
            false
        }
    }) {
        if let Some(LexUnit::Block(elif_block)) = block_stream.next() {
            let (elif_condition, elif_body) =
                parse_condition_body(elif_block, &mut elif_func, arena);
            conditions.push(ConditionBody {
                condition: elif_condition,
                body: elif_body,
            })
        } else {
            panic!("Invariant Violated")
        }
    }

    let mut else_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(
            Some(Token::Block(BlockType::Else)),
            it.next(),
            "If the function is called, should be parsing an else block"
        );
        assert_eq!(
            Some(Token::Symbol(Symbol::Colon)),
            it.next(),
            "An else block shouldn't have a condition"
        );
    };
    let else_body = if block_stream.peek().map_or(false, |lex_unit| {
        if let LexUnit::Block(block) = lex_unit {
            block.kind == BlockType::Else
        } else {
            false
        }
    }) {
        if let Some(LexUnit::Block(else_block)) = block_stream.next() {
            let ((), else_body) = parse_condition_body(else_block, &mut else_func, arena);
            Some(else_body)
        } else {
            panic!("Invariant Violated")
        }
    } else {
        None
    };

    Branch::IfElseBlock {
        conditions,
        last: else_body,
    }
}

/// Parse a line and its connected blocks
fn parse_condition_body<F, R>(
    block: Block,
    condition_parse: &mut F,
    arena: &mut Arena<Eval>,
) -> (R, ExpRef)
where
    F: FnMut(&mut dyn Iterator<Item = Token>, &mut Arena<Eval>) -> R,
{
    let Block {
        mut line, children, ..
    } = block;
    let mut iter = line.into_iter();
    let (condition, body) = if children.is_empty() {
        let condition = condition_parse(&mut iter, arena);
        let body = parse_to(
            iter.collect::<Vec<_>>().as_mut_slice(),
            ParseTarget::End,
            arena,
        );
        (condition, body)
    } else {
        let condition = condition_parse(&mut iter, arena);
        assert!(
            iter.next().is_none(),
            "Parsing a while loop with a body, colon should be end of my line"
        );
        let mut child_iter = children.into_iter().peekable();
        let mut body_pieces = Vec::new();
        while let Some(_) = child_iter.peek() {
            body_pieces.push(parse_lex_unit(&mut child_iter, arena));
        }
        let body = Expression::Joiner {
            expressions: body_pieces,
        };
        (condition, body)
    };
    (condition, arena.insert(Eval::Expression(body)))
}

fn parse_for(block: Block, arena: &mut Arena<Eval>) -> Branch {
    assert_eq!(
        BlockType::For,
        block.kind,
        "If the function is called, should be parsing a for loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(Some(Token::Block(BlockType::For)), it.next());
        unimplemented!()
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func, arena);
    //    let condition = arena.insert(Eval::Expression(condition));
    unimplemented!()
}

fn parse_compare(block: Block, arena: &mut Arena<Eval>) -> Branch {
    assert_eq!(
        BlockType::Cmp,
        block.kind,
        "If the function is called, should be parsing a for loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>, arena: &mut Arena<Eval>| {
        assert_eq!(Some(Token::Block(BlockType::Cmp)), it.next());
        unimplemented!()
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func, arena);
    //    let condition = arena.insert(Eval::Expression(condition));
    unimplemented!()
}

/// Parse a branching block (type of [Branch](crate::parser::branch::Branch))
fn parse_branch<I>(block: Block, block_stream: &mut I, arena: &mut Arena<Eval>) -> Branch
where
    I: Iterator<Item = LexUnit>,
{
    let block = match block.kind {
        BlockType::If => parse_if_else(block, block_stream, arena),
        BlockType::While => parse_while(block, arena),
        BlockType::For => parse_for(block, arena),
        BlockType::Cmp => parse_compare(block, arena),
        BlockType::Else | BlockType::ElseIf => panic!("Parsed else without if"),
    };

    block
}

fn absorb_whitespace<I>(tokens: &mut I) -> bool
where
    I: Iterator<Item = Token>,
{
    let mut tokens = tokens.peekable();
    let mut found = false;
    while tokens
        .peek()
        .map_or(false, |token| token == &Token::WhiteSpace)
    {
        found = true;
    }
    found
}

fn slice_whitespace(tokens: &mut [Token]) -> &mut [Token] {
    let mut spaces = 0;
    while let Some(Token::WhiteSpace) = tokens.get(spaces) {
        spaces += 1;
    }
    &mut tokens[spaces..]
}

fn parse_identifier<I>(tokens: &mut I, arena: &mut Arena<Eval>) -> Expression
where
    I: Iterator<Item = Token>,
{
    absorb_whitespace(tokens);
    let reference = if let Some(Token::Reference(name)) = tokens.next() {
        name
    } else {
        panic!()
    };

    let mut expr = Reference { target: reference };

    while let Some(token) = tokens.next() {
        if let Token::Symbol(Symbol::Period) = token {
            if let Some(Token::Reference(name)) = tokens.next() {
                expr = FieldAccess {
                    target: arena.insert(Eval::Expression(expr)),
                    name,
                }
            }
        } else if let Some(Token::WhiteSpace) = tokens.next() {
            absorb_whitespace(tokens);
            break;
        }
    }
    assert_eq!(tokens.next(), None);

    expr
}

fn parse_statement(mut tokens: Vec<Token>, arena: &mut Arena<Eval>) -> Expression {
    let mut is_assignment = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Symbol(symbol) = token {
                Some(*symbol)
            } else {
                None
            }
        })
        .enumerate()
        .find(|&(_, token)| token == Symbol::Equals || token == Symbol::ConditionalAssignment);

    if let Some((index, kind)) = is_assignment {
        let mut is_let = false;
        let mut tokens = tokens.into_boxed_slice();

        let (mut lhs, mut rhs) = tokens.split_at_mut(index);
        rhs = &mut rhs[1..];

        if let Some(Token::Let) = lhs.first() {
            lhs = &mut lhs[1..];
            is_let = true;
        }
        // Todo: Remove cloning
        let mut lhs_iter = lhs.iter().cloned();
        let target = parse_identifier(&mut lhs_iter, arena);
        let target = arena.insert(Eval::Expression(target));
        let mut rhs_iter = rhs.iter().cloned();
        let expression = Eval::Expression(parse_to(
            rhs_iter.collect::<Vec<_>>().as_mut_slice(),
            ParseTarget::End,
            arena,
        ));
        let expression = arena.insert(expression);
        let mut flags = AssignmentFlags::empty();
        if is_let {
            flags |= AssignmentFlags::LET;
        }
        if kind == ConditionalAssignment {
            flags |= AssignmentFlags::CONDITIONAL;
        }

        Assigment {
            target,
            expression,
            flags,
        }
    } else {
        parse_to(tokens.as_mut_slice(), ParseTarget::End, arena)
    }
}

fn operator_split<C>(mut tokens: &mut [Token], index: usize, construct: C, arena: &mut Arena<Eval>) -> Expression
where
    C: Fn(ExpRef, ExpRef) -> Expression,
{
    let (lsplit, rsplit) = tokens.split_at_mut(index);
    let lhs = Eval::Expression(parse_to(lsplit, ParseTarget::End, arena));
    let lhs = arena.insert(lhs);
    let rhs = Eval::Expression(parse_to(rsplit, ParseTarget::End, arena));
    let rhs = arena.insert(rhs);
    construct(lhs, rhs)
}

enum ParseTarget {
    Colon,
    End,
}

const EQUALITY_OPERATORS: &[Token] = &[Token::Symbol(Symbol::DoubleEquals), Token::Symbol(Symbol::NotEquals), Token::Symbol(Symbol::Less), Token::Symbol(Symbol::LessEquals), Token::Symbol(Symbol::Greater), Token::Symbol(Symbol::GreaterEquals)];
const ADDITION_OPERATORS: &[Token] = &[Token::Symbol(Symbol::Plus), Token::Symbol(Symbol::Minus)];
const MULTIPLICATION_OPERATORS: &[Token] =  &[Token::Symbol(Symbol::Star), Token::Symbol(Symbol::Slash), Token::Symbol(Symbol::DoubleSlash), Token::Symbol(Symbol::Percent)];

/// Parse an expression from tokens until a specified token is reached (consumes said token)
fn parse_to(mut tokens: &mut [Token], target: ParseTarget, arena: &mut Arena<Eval>) -> Expression {
    if let ParseTarget::Colon = target {
        let mut split = tokens.splitn_mut(2, |elem| elem == &Token::Symbol(Symbol::Colon));
        tokens = split.next().expect("Missing Colon");
        let after_colon = split.next().expect("Missing Colon");
        assert!(after_colon.iter().all(|elem| elem == &Token::WhiteSpace));
    }
    if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::SemiColon)) {
        let (lhs, rhs) = tokens.split_at_mut(index);
        assert!(rhs.iter().all(|token| token == &Token::WhiteSpace));
        let lhs = parse_to(lhs, ParseTarget::End, arena);
        return Nullify { expr: arena.insert(Eval::Expression(lhs)) }
    }
    if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::Elvis)) {
        operator_split(tokens, index, |lhs, rhs| Expression::Elvis {lhs, rhs}, arena)
    } else if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::Or)) {
        operator_split(tokens, index, |lhs, rhs| Expression::Or {lhs, rhs}, arena)
    } else if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::And)) {
        operator_split(tokens, index, |lhs, rhs| Expression::And {lhs, rhs}, arena)
    } else if let Some((index, kind)) = tokens.iter().enumerate().find(|token| EQUALITY_OPERATORS.contains(token.1)) {
        let operation = match kind {
            Token::Symbol(Symbol::DoubleEquals) => Compare::Equal,
            Token::Symbol(Symbol::NotEquals) => Compare::NotEqual,
            Token::Symbol(Symbol::GreaterEquals) => Compare::GreaterEqual,
            Token::Symbol(Symbol::Greater) => Compare::Greater,
            Token::Symbol(Symbol::LessEquals) => Compare::LessEqual,
            Token::Symbol(Symbol::Less) => Compare::Less,
            _ => panic!()
        };
        operator_split(tokens, index, |lhs, rhs| Expression::Compare {lhs, rhs, operation }, arena)
    } else if let Some((index, kind)) = tokens.iter().enumerate().find(|&(_, token)| ADDITION_OPERATORS.contains(token)) {
        let construct: Box<dyn Fn(ExpRef, ExpRef) -> Expression> =
            if kind == &Token::Symbol(Symbol::Plus) {
                Box::new(|lhs, rhs| Addition { lhs, rhs })
            } else if kind == &Token::Symbol(Symbol::Minus) {
                Box::new(|lhs, rhs| Subtract { lhs, rhs })
            } else {
                panic!("{:?}", kind)
            };
        operator_split(tokens, index, construct, arena)
    } else if let Some((index, kind)) = tokens.iter().enumerate().find(|&(_, token)| MULTIPLICATION_OPERATORS.contains(token)) {
        let construct: Box<dyn Fn(ExpRef, ExpRef) -> Expression> =
            if kind == &Token::Symbol(Symbol::Star) {
                Box::new(|lhs, rhs| Addition { lhs, rhs })
            } else if kind == &Token::Symbol(Symbol::Slash) {
                Box::new(|lhs, rhs| Subtract { lhs, rhs })
            } else if kind == &Token::Symbol(Symbol::DoubleSlash) {
                Box::new(|lhs, rhs| Subtract { lhs, rhs })
            } else if kind == &Token::Symbol(Symbol::Percent) {
                Box::new(|lhs, rhs| Subtract { lhs, rhs })
            } else {
                panic!("{:?}", kind)
            };
        operator_split(tokens, index, construct, arena)
    } else if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::DoubleStar)) {
        operator_split(tokens, index, |lhs, rhs| Expression::Exponent {lhs, rhs}, arena)
    } else if let Some(index) = tokens.iter().position(|token| token == &Token::Symbol(Symbol::ExclamationPoint)) {
        let (lhs, rhs) = tokens.split_at_mut(index);
        assert!(lhs.iter().all(|token| token == &Token::WhiteSpace));
        let expr = parse_to(rhs, ParseTarget::End, arena);
        Negate { operand: arena.insert(Eval::Expression(expr)) }
    } else {
        unimplemented!()
    }
}
