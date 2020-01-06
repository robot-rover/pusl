//! The parser takes the token hierarchy produced by the lexer and creates an abstract syntax tree.
//! This is where grammatical errors are caught (lexer catches syntax errors).
//! This data is taken in by the linearization engine before being executed.

use crate::lexer::peek_while::peek_while;
use crate::lexer::token::Symbol::ConditionalAssignment;
use crate::lexer::token::{Block, BlockType, LexUnit, Literal, Symbol, Token, Keyword};
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::AssignmentFlags;
use crate::parser::expression::Expression;
use crate::parser::expression::Compare;
use crate::parser::expression::Expression::{Assigment, FieldAccess, Reference, Subtract, Addition, Nullify, Negate, FunctionCall};
use generational_arena::{Arena, Index};
use std::io::Read;
use std::collections::LinkedList;
use crate::parser::InBetween::{Lexeme, Parsed};

pub mod branch;
pub mod expression;

/// All structures in the Abstract Syntax Tree
/// Branch and Expression or different because Branch will affect execution flow
/// Expression executes in a deterministic order
enum Eval {
    Expression(Expression),
    Branch(Branch),
}

static mut STATIC_ARENA: Option<Arena<Eval>> = None;

type ExpRef = Index;

fn parse<I>(source: I) -> Eval
    where
        I: IntoIterator<Item=LexUnit>,
{
    // Todo: Remove this
//    let arena: Arena<Eval> = Arena::new();
    unsafe {STATIC_ARENA = Some(Arena::new())};
    let iter = source.into_iter();
    let expr_list = Vec::new();
    while let Some(unit) = iter.next() {
        parse_lex_unit(unit, unsafe { &mut STATIC_ARENA.unwrap() });
    }

    unimplemented!()
}

fn parse_lex_unit<I>(unit: LexUnit, arena: &mut Arena<Eval>) -> ExpRef {
    match unit {
        LexUnit::Block(block) => parse_branch(block, unit_stream, arena),
        LexUnit::Statement(tokens) => parse_statement(tokens, arena)
    }
}

/// Parse a while loop
fn parse_while(block: Block, arena: &mut Arena<Eval>) -> Branch {
    assert_eq!(
        BlockType::While,
        block.kind,
        "If the function is called, should be parsing a while loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
        assert_eq!(Some(Token::Block(BlockType::While)), it.next());
        parse_expression(it, arena)
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func, arena);
    Branch::WhileLoop { condition, body }
}

/// Parse a group of if, else if, ..., else blocks
fn parse_if_else<I>(if_block: Block, block_stream: &mut I, arena: &mut Arena<Eval>) -> Branch
    where
        I: Iterator<Item=LexUnit>,
{
    let mut block_stream = block_stream.peekable();
    let mut conditions = Vec::<ConditionBody>::new();

    let mut if_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
        assert_eq!(
            Some(Token::Block(BlockType::If)),
            it.next(),
            "If the function is called, should be parsing an if block"
        );
        parse_expression(it, arena)
    };
    let (if_condition, if_body) = parse_condition_body(if_block, &mut if_func, arena);
    conditions.push(ConditionBody {
        condition: if_condition,
        body: if_body,
    });

    let mut elif_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
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
        parse_expression(it, arena)
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

    let mut else_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
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
        F: FnMut(&mut dyn Iterator<Item=Token>, &mut Arena<Eval>) -> R,
{
    let Block {
        mut line, children, ..
    } = block;
    let mut iter = line.into_iter();
    let (condition, body) = if children.is_empty() {
        // Todo: this isn't necessary
        let mut found_colon = false;
        let condition = condition_parse(&mut iter.by_ref().take_while(|token| {
            found_colon = token == &Token::Symbol(Symbol::Colon);
            !found_colon
        }), arena);
        assert!(found_colon);
        let body = parse_expression(&mut iter, arena);
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
        (condition, arena.insert(Eval::Expression(body)))
    };
    (condition, body)
}

fn parse_for(block: Block, arena: &mut Arena<Eval>) -> Branch {
    assert_eq!(
        BlockType::For,
        block.kind,
        "If the function is called, should be parsing a for loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
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
    let mut condition_func = |it: &mut dyn Iterator<Item=Token>, arena: &mut Arena<Eval>| {
        assert_eq!(Some(Token::Block(BlockType::Cmp)), it.next());
        unimplemented!()
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func, arena);
    //    let condition = arena.insert(Eval::Expression(condition));
    unimplemented!()
}

/// Parse a branching block (type of [Branch](crate::parser::branch::Branch))
fn parse_branch<I>(block: Block, block_stream: &mut I, arena: &mut Arena<Eval>) -> ExpRef
    where
        I: Iterator<Item=LexUnit>,
{
    let block = match block.kind {
        BlockType::If => parse_if_else(block, block_stream, arena),
        BlockType::While => parse_while(block, arena),
        BlockType::For => parse_for(block, arena),
        BlockType::Cmp => parse_compare(block, arena),
        BlockType::Else | BlockType::ElseIf => panic!("Parsed else without if"),
    };

    arena.insert(Eval::Branch(block))
}

fn parse_identifier<I>(tokens: &mut I, arena: &mut Arena<Eval>) -> Expression
    where
        I: Iterator<Item=Token>,
{
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
                    name: arena.insert(Eval::Expression(Expression::Reference { target: name })),
                }
            }
        }
    }
    assert_eq!(tokens.next(), None);

    expr
}

fn parse_statement(mut tokens: Vec<Token>, arena: &mut Arena<Eval>) -> ExpRef {
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

        if let Some(Token::Keyword(Keyword::Let)) = lhs.first() {
            lhs = &mut lhs[1..];
            is_let = true;
        }
        // Todo: Remove cloning
        let mut lhs_iter = lhs.iter().cloned();
        let target = parse_identifier(&mut lhs_iter, arena);
        let target = arena.insert(Eval::Expression(target));
        let mut rhs_iter = rhs.iter().cloned();
        let expression = parse_expression(&mut rhs_iter, arena);
        let mut flags = AssignmentFlags::empty();
        if is_let {
            flags |= AssignmentFlags::LET;
        }
        if kind == ConditionalAssignment {
            flags |= AssignmentFlags::CONDITIONAL;
        }

        let expr = Assigment {
            target,
            expression,
            flags,
        };
        arena.insert(Eval::Expression(expr))
    } else {
        parse_expression(&mut tokens.into_iter(), arena)
    }
}

const EQUALITY_OPERATORS: &[Token] = &[Token::Symbol(Symbol::DoubleEquals), Token::Symbol(Symbol::NotEquals), Token::Symbol(Symbol::Less), Token::Symbol(Symbol::LessEquals), Token::Symbol(Symbol::Greater), Token::Symbol(Symbol::GreaterEquals)];
const ADDITION_OPERATORS: &[Token] = &[Token::Symbol(Symbol::Plus), Token::Symbol(Symbol::Minus)];
const MULTIPLICATION_OPERATORS: &[Token] = &[Token::Symbol(Symbol::Star), Token::Symbol(Symbol::Slash), Token::Symbol(Symbol::DoubleSlash), Token::Symbol(Symbol::Percent)];

enum InBetween {
    Lexeme(Token),
    Parsed(ExpRef),
}

// Call only after initial parenthesis has been consumed
fn parse_inside_parenthesis<I: ?Sized>(tokens: &mut I, arena: &mut Arena<Eval>) -> ExpRef
    where I: Iterator<Item=Token> {
    let mut level = 1;
    let mut take_while = tokens.take_while(|token| {
        match token {
            Token::Symbol(Symbol::OpenParenthesis) => level += 1,
            Token::Symbol(Symbol::CloseParenthesis) => level -= 1,
            _ => {}
        };

        level > 0
    });

    parse_expression(&mut take_while, arena)
}

// Returns a Function Call expression with a null target field
fn parse_function_parenthesis<I: ?Sized>(tokens: &mut I, arena: &mut Arena<Eval>) -> Expression
    where I: Iterator<Item=Token> {
    let mut next = true;
    let mut arguments = Vec::new();
    while next {
        let mut level = 0;
        let mut take_while = tokens.take_while(|token| {
            match token {
                Token::Symbol(Symbol::OpenParenthesis) => level += 1,
                Token::Symbol(Symbol::CloseParenthesis) => {
                    level -= 1;
                    if level < 0 {
                        next = false;
                        return false;
                    }
                }
                Token::Symbol(Symbol::Comma) => {
                    if level == 0 { return false; }
                }
                _ => {}
            };

            true
        });

        let expr = parse_expression(&mut take_while, arena);
        arguments.push(expr);
    }

    let target = arena.insert(Eval::Expression(Expression::Literal { value: Literal::Null }));

    FunctionCall { target, arguments }
}

fn parser_pass<I>(progress: I, targets: Vec<(Token, Box<dyn Fn(ExpRef, ExpRef) -> Expression>)>, arena: &mut Arena<Eval>) -> Vec<InBetween>
    where I: IntoIterator<Item=InBetween> {
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = if let Lexeme(token) = next {
            if let Some((_, func)) = targets.iter().find(|(target, _)| target == &token) {
                let lhs_exp = if let Some(Parsed(exp_ref)) = result.pop() {
                    exp_ref
                } else {
                    panic!()
                };
                let rhs_exp = if let Some(Parsed(exp_ref)) = iter.next() {
                    exp_ref
                } else {
                    panic!()
                };
                let expr = func(lhs_exp, rhs_exp);
                Parsed(arena.insert(Eval::Expression(expr)))
            } else {
                Lexeme(token)
            }
        } else {
            next
        };
        result.push(next_between)
    }

    result
}

// Function Call parenthesis are parsed before their target can be determined (field access needs to come first)
// This function removes the null target and sets the target as the symbol that comes before the function call
fn parser_pass_function_call<I>(progress: I, arena: &mut Arena<Eval>) -> Vec<InBetween>
    where I: IntoIterator<Item=InBetween> {
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = if let Parsed(exp_ref) = next {
            if let Some(Eval::Expression(Expression::FunctionCall { target, .. })) = arena.get_mut(exp_ref) {
                let new_target = if let Some(Parsed(exp_ref)) = result.pop() {
                    exp_ref
                } else {
                    panic!()
                };
                *target = new_target;
            }
            Parsed(exp_ref)
        } else {
            next
        };

        result.push(next_between);
    }

    result
}

fn parser_pass_negate<I>(progress: I, arena: &mut Arena<Eval>) -> Vec<InBetween>
    where I: IntoIterator<Item=InBetween> {
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = if let Lexeme(Token::Symbol(Symbol::ExclamationPoint)) = next {
            let exp_ref = if let Some(Parsed(exp_ref)) = iter.next() {
                exp_ref
            } else {
                panic!()
            };
            let expr = Expression::Negate { operand: exp_ref };
            Parsed(arena.insert(Eval::Expression(expr)))
        } else {
            next
        };

        result.push(next_between);
    }

    result
}

/// Parse an expression from tokens until a specified token is reached (consumes said token)
fn parse_expression<I: ?Sized>(mut tokens: &mut I, arena: &mut Arena<Eval>) -> ExpRef
    where I: Iterator<Item=Token> {
    let mut between = Vec::new();
    while let Some(token) = tokens.next() {
        let next = match token {
            Token::Literal(literal) => Parsed(arena.insert(Eval::Expression(Expression::Literal { value: literal }))),
            Token::Reference(name) => Parsed(arena.insert(Eval::Expression(Expression::Reference { target: name }))),
            Token::Symbol(Symbol::OpenParenthesis) => {
                if let Some(Parsed(_)) = between.last() {
                    let expr = parse_function_parenthesis(tokens, arena);
                    Parsed(arena.insert(Eval::Expression(expr)))
                } else {
                    Parsed(parse_inside_parenthesis(tokens, arena))
                }
            }
            other_token => Lexeme(other_token)
        };
        between.push(next);
    }

    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::Period), Box::new(|lhs, rhs| Expression::FieldAccess { target: lhs, name: rhs }))
    ], arena);
    between = parser_pass_function_call(between, arena);
    between = parser_pass_negate(between, arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::DoubleStar), Box::new(|lhs, rhs| Expression::Exponent { lhs, rhs }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::Star), Box::new(|lhs, rhs| Expression::Multiply { lhs, rhs })),
        (Token::Symbol(Symbol::Slash), Box::new(|lhs, rhs| Expression::Divide { lhs, rhs })),
        (Token::Symbol(Symbol::DoubleSlash), Box::new(|lhs, rhs| Expression::DivideTruncate { lhs, rhs })),
        (Token::Symbol(Symbol::Percent), Box::new(|lhs, rhs| Expression::Modulus { lhs, rhs }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::Plus), Box::new(|lhs, rhs| Expression::Addition { lhs, rhs })),
        (Token::Symbol(Symbol::Minus), Box::new(|lhs, rhs| Expression::Subtract { lhs, rhs }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::And), Box::new(|lhs, rhs| Expression::And { lhs, rhs }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::Or), Box::new(|lhs, rhs| Expression::Or { lhs, rhs }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::DoubleEquals), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::Equal })),
        (Token::Symbol(Symbol::Less), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::Less })),
        (Token::Symbol(Symbol::LessEquals), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::LessEqual })),
        (Token::Symbol(Symbol::Greater), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::Greater })),
        (Token::Symbol(Symbol::GreaterEquals), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::GreaterEqual })),
        (Token::Symbol(Symbol::NotEquals), Box::new(|lhs, rhs| Expression::Compare { lhs, rhs, operation: Compare::NotEqual }))
    ], arena);
    between = parser_pass(between, vec![
        (Token::Symbol(Symbol::Elvis), Box::new(|lhs, rhs| Expression::Elvis { lhs, rhs }))
    ], arena);
    let expr = if let Some(Parsed(exp_ref)) = between.pop() {
        exp_ref
    } else {
        panic!()
    };
    assert!(between.is_empty());
    expr
}
