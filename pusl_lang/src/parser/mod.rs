//! The parser takes the token hierarchy produced by the lexer and creates an abstract syntax tree.
//! This is where grammatical errors are caught (lexer catches syntax errors).
//! This data is taken in by the linearization engine before being executed.

use crate::lexer::token::{Block, BlockType, Keyword, LexUnit, Symbol, Token};
use crate::parser::branch::{Branch, ConditionBody};
use crate::parser::expression::Compare;
use crate::parser::expression::Expression;
use crate::parser::expression::{AssignAccess, AssignmentFlags};
use crate::parser::InBetween::{Lexeme, Parsed};
use serde::{Deserialize, Serialize};
use std::iter::Peekable;

pub mod branch;
pub mod expression;

/// All structures in the Abstract Syntax Tree
/// Branch and Expression or different because Branch will affect execution flow
/// Expression executes in a deterministic order
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Eval {
    Expression(Expression),
    Branch(Branch),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ParsedFile {
    pub expr: ExpRef,
    pub imports: Vec<Import>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: String,
}

pub type ExpRef = Box<Eval>;

pub fn parse<I>(source: I) -> ParsedFile
where
    I: IntoIterator<Item = LexUnit>,
{
    let mut iter = source.into_iter().peekable();
    let mut imports = Vec::new();
    while let Some(LexUnit::Statement(tokens)) = iter.peek() {
        if let Some(&Token::Keyword(Keyword::Import)) = tokens.first() {
            if let Some(LexUnit::Statement(tokens)) = iter.next() {
                let import = parse_import(tokens);
                imports.push(import);
            } else {
                panic!("Invariant");
            }
        } else {
            break;
        }
    }
    let mut expr_list = Vec::new();
    while let Some(unit) = iter.next() {
        let expr = parse_lex_unit(unit, &mut iter);
        expr_list.push(expr);
    }

    let expr = Box::new(Eval::Expression(Expression::Joiner {
        expressions: expr_list,
    }));

    ParsedFile { expr, imports }
}

fn parse_import<I>(tokens: I) -> Import
where
    I: IntoIterator<Item = Token>,
{
    let mut iter = tokens.into_iter().peekable();
    assert_eq!(Some(Token::Keyword(Keyword::Import)), iter.next());
    let mut path = Vec::new();
    while let Some(token) = iter.next() {
        if let Token::Reference(name) = token {
            path.push(name);
            match iter.next() {
                Some(Token::Symbol(Symbol::Period)) => {}
                Some(Token::Keyword(Keyword::As)) => {
                    break;
                }
                _ => panic!(),
            }
        } else {
            panic!("Invalid Import")
        }
    }

    let alias = if let Some(Token::Reference(alias)) = iter.next() {
        alias
    } else {
        panic!()
    };

    assert_eq!(iter.next(), None);

    Import { path, alias }
}

fn parse_lex_unit<I>(unit: LexUnit, stream: &mut Peekable<I>) -> ExpRef
where
    I: Iterator<Item = LexUnit>,
{
    match unit {
        LexUnit::Block(block) => parse_branch(block, stream),
        LexUnit::Statement(tokens) => parse_statement(tokens),
    }
}

/// Parse a while loop
fn parse_while(block: Block) -> Branch {
    assert_eq!(
        BlockType::While,
        block.kind,
        "If the function is called, should be parsing a while loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(Some(Token::Block(BlockType::While)), it.next());
        parse_expression(it)
    };
    let (condition, body) = parse_condition_body(block, &mut condition_func);
    Branch::WhileLoop { condition, body }
}

fn parse_try<I>(try_block: Block, block_stream: &mut Peekable<I>) -> Branch
where
    I: Iterator<Item = LexUnit>,
{
    let mut try_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(
            Some(Token::Block(BlockType::Try)),
            it.next(),
            "If the function is called, should be parsing an try block"
        );
        assert_eq!(None, it.next(), "Try block should have empty condition");
    };
    let (_, try_body) = parse_condition_body(try_block, &mut try_func);

    let mut yoink_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(
            Some(Token::Block(BlockType::Yoink)),
            it.next(),
            "If the function is called, should be parsing an yoink block"
        );
        let mut expr_tokens = it.collect::<Vec<_>>();
        let variable = if let Some(Token::Reference(name)) = expr_tokens.pop() {
            name
        } else {
            panic!("No variable name in yoink condition");
        };
        let expr = parse_expression(&mut expr_tokens.into_iter());
        (expr, variable)
    };

    let ((filter_expr, error_variable), yoink_body) =
        if let Some(LexUnit::Block(block)) = block_stream.next() {
            assert_eq!(
                block.kind,
                BlockType::Yoink,
                "yoink Block should follow try block"
            );
            parse_condition_body(block, &mut yoink_func)
        } else {
            panic!("yoink Block should follow try block");
        };

    Branch::TryBlock {
        try_body,
        filter_expr,
        error_variable,
        yoink_body,
    }
}

/// Parse a group of if, else if, ..., else blocks
fn parse_if_else<I>(if_block: Block, block_stream: &mut Peekable<I>) -> Branch
where
    I: Iterator<Item = LexUnit>,
{
    let mut conditions = Vec::<ConditionBody>::new();

    let mut if_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(
            Some(Token::Block(BlockType::If)),
            it.next(),
            "If the function is called, should be parsing an if block"
        );
        parse_expression(it)
    };
    let (if_condition, if_body) = parse_condition_body(if_block, &mut if_func);
    conditions.push(ConditionBody {
        condition: if_condition,
        body: if_body,
    });

    let mut elif_func = |it: &mut dyn Iterator<Item = Token>| {
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
        parse_expression(it)
    };
    while block_stream.peek().map_or(false, |lex_unit| {
        if let LexUnit::Block(block) = lex_unit {
            block.kind == BlockType::ElseIf
        } else {
            false
        }
    }) {
        if let Some(LexUnit::Block(elif_block)) = block_stream.next() {
            let (elif_condition, elif_body) = parse_condition_body(elif_block, &mut elif_func);
            conditions.push(ConditionBody {
                condition: elif_condition,
                body: elif_body,
            })
        } else {
            panic!("Invariant Violated")
        }
    }

    let mut else_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(
            Some(Token::Block(BlockType::Else)),
            it.next(),
            "If the function is called, should be parsing an else block"
        );
        assert_eq!(None, it.next(), "An else block shouldn't have a condition");
    };
    let else_body = if block_stream.peek().map_or(false, |lex_unit| {
        if let LexUnit::Block(block) = lex_unit {
            block.kind == BlockType::Else
        } else {
            false
        }
    }) {
        if let Some(LexUnit::Block(else_block)) = block_stream.next() {
            let ((), else_body) = parse_condition_body(else_block, &mut else_func);
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
fn parse_condition_body<F, R>(block: Block, condition_parse: &mut F) -> (R, ExpRef)
where
    F: FnMut(&mut dyn Iterator<Item = Token>) -> R,
{
    let Block { line, children, .. } = block;
    let (condition, body) = if children.is_empty() {
        panic!("Block has no children")
    } else {
        let mut found_colon = false;
        let condition = condition_parse(&mut line.into_iter().take_while(|token| {
            found_colon = token == &Token::Symbol(Symbol::Colon);
            !found_colon
        }));
        assert!(
            found_colon,
            "Parsing a while loop with a body, colon should be end of my line"
        );
        let mut child_iter = children.into_iter().peekable();
        let mut body_pieces = Vec::new();
        while let Some(next) = child_iter.next() {
            body_pieces.push(parse_lex_unit(next, &mut child_iter));
        }
        let body = Expression::Joiner {
            expressions: body_pieces,
        };
        (condition, Box::new(Eval::Expression(body)))
    };
    (condition, body)
}

fn parse_for(block: Block) -> Branch {
    assert_eq!(
        BlockType::For,
        block.kind,
        "If the function is called, should be parsing a for loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(Some(Token::Block(BlockType::For)), it.next());
        let var_name = if let Some(Token::Reference(var_name)) = it.next() {
            var_name
        } else {
            panic!("For Loop expects variable name")
        };
        assert_eq!(
            Some(Token::Keyword(Keyword::In)),
            it.next(),
            "For loop expects 'in' keyword"
        );
        let generator_expression = parse_expression(it);
        (var_name, generator_expression)
    };
    let ((var_name, generator_expression), body) = parse_condition_body(block, &mut condition_func);

    Branch::ForLoop {
        variable: var_name,
        iterable: generator_expression,
        body,
    }
}

fn parse_compare(block: Block) -> Branch {
    assert_eq!(
        BlockType::Cmp,
        block.kind,
        "If the function is called, should be parsing a for loop"
    );
    let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
        assert_eq!(Some(Token::Block(BlockType::Cmp)), it.next());
        unimplemented!()
    };
    let (_, _) = parse_condition_body(block, &mut condition_func);
    //    let condition = arena.insert(Eval::Expression(condition));
    unimplemented!()
}

/// Parse a branching block (type of [Branch](crate::parser::branch::Branch))
fn parse_branch<I>(block: Block, block_stream: &mut Peekable<I>) -> ExpRef
where
    I: Iterator<Item = LexUnit>,
{
    let block = match block.kind {
        BlockType::If => Eval::Branch(parse_if_else(block, block_stream)),
        BlockType::While => Eval::Branch(parse_while(block)),
        BlockType::For => Eval::Branch(parse_for(block)),
        BlockType::Cmp => Eval::Branch(parse_compare(block)),
        BlockType::Function => Eval::Expression(parse_function_declaration(block)),
        BlockType::Try => Eval::Branch(parse_try(block, block_stream)),
        BlockType::Else | BlockType::ElseIf => panic!("Parsed else without if"),
        BlockType::Yoink => panic!("Parsed yoink without try"),
    };

    Box::new(block)
}

fn parse_function_declaration(block: Block) -> Expression {
    let mut declaration_func = |it: &mut dyn Iterator<Item = Token>| {
        let line = it.collect::<Vec<_>>();
        let is_assignment = line
            .iter()
            .enumerate()
            .filter_map(|(index, token)| {
                if let Token::Symbol(symbol) = token {
                    Some((index, *symbol))
                } else {
                    None
                }
            })
            .find(|&(_, token)| token == Symbol::Equals || token == Symbol::ConditionalAssignment);

        if let Some((index, kind)) = is_assignment {
            let mut is_let = false;
            let mut tokens = line.into_boxed_slice();

            let (mut lhs, mut rhs) = tokens.split_at_mut(index);
            rhs = &mut rhs[1..];

            if let Some(Token::Keyword(Keyword::Let)) = lhs.first() {
                lhs = &mut lhs[1..];
                is_let = true;
            }
            // Todo: Remove cloning
            let mut lhs_iter = lhs.iter().cloned();
            let target = parse_identifier(&mut lhs_iter);

            let mut rhs_iter = rhs.iter().cloned();
            assert_eq!(Some(Token::Keyword(Keyword::Fn)), rhs_iter.next());
            let mut open_symbol = rhs_iter.next().unwrap();
            let binds = if let Token::Symbol(Symbol::OpenSquareBracket) = open_symbol {
                let binds = parse_function_parameters(&mut rhs_iter, Symbol::CloseSquareBracket);
                open_symbol = rhs_iter.next().unwrap();
                binds
            } else {
                Vec::new()
            };
            assert_eq!(Token::Symbol(Symbol::OpenParenthesis), open_symbol);
            let parameters = parse_function_parameters(&mut rhs_iter, Symbol::CloseParenthesis);
            assert_eq!(None, rhs_iter.next());

            let mut flags = AssignmentFlags::empty();
            if is_let {
                flags |= AssignmentFlags::LET;
            }
            if kind == Symbol::ConditionalAssignment {
                flags |= AssignmentFlags::CONDITIONAL;
            }
            (target, flags, binds, parameters)
        } else {
            panic!("Function declaration without assignment")
        }
    };
    let ((target, flags, binds, params), body) = parse_condition_body(block, &mut declaration_func);
    let decl_expr = Expression::FunctionDeclaration {
        binds,
        params,
        body,
    };
    let decl_expr = Box::new(Eval::Expression(decl_expr));

    Expression::Assigment {
        target,
        expression: decl_expr,
        flags,
    }
}

fn parse_identifier<I>(tokens: &mut I) -> AssignAccess
where
    I: Iterator<Item = Token>,
{
    //Todo: Improve This
    let mut tokens = tokens.collect::<Vec<_>>();
    let name = if let Some(Token::Reference(name)) = tokens.pop() {
        name
    } else {
        panic!()
    };

    let ident = if tokens.is_empty() {
        AssignAccess::Reference { name }
    } else {
        let mut name_stack = Vec::new();
        let mut this_base = false;
        while let Some(Token::Symbol(Symbol::Period)) = tokens.pop() {
            match tokens.pop() {
                Some(Token::Reference(name)) => name_stack.push(name),
                Some(Token::Keyword(Keyword::This)) => {
                    this_base = true;
                    assert!(tokens.is_empty(), "this is not a valid field name")
                }
                // Some(Token::Keyword())
                other => panic!("Invalid Identifier: {:?}", other),
            }
        }
        let mut target = if this_base {
            Box::new(Eval::Expression(Expression::ThisReference))
        } else {
            Box::new(Eval::Expression(Expression::Reference {
                target: name_stack.pop().unwrap(),
            }))
        };
        while let Some(name) = name_stack.pop() {
            target = Box::new(Eval::Expression(Expression::FieldAccess { target, name }))
        }
        AssignAccess::Field { target, name }
    };
    assert!(tokens.is_empty());

    ident
}

fn parse_statement(tokens: Vec<Token>) -> ExpRef {
    let is_assignment = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if let Token::Symbol(symbol) = token {
                Some((index, *symbol))
            } else {
                None
            }
        })
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
        let target = parse_identifier(&mut lhs_iter);
        let mut rhs_iter = rhs.iter().cloned();
        let expression = parse_expression(&mut rhs_iter);
        let mut flags = AssignmentFlags::empty();
        if is_let {
            flags |= AssignmentFlags::LET;
        }
        if kind == Symbol::ConditionalAssignment {
            flags |= AssignmentFlags::CONDITIONAL;
        }

        let expr = Expression::Assigment {
            target,
            expression,
            flags,
        };
        Box::new(Eval::Expression(expr))
    } else {
        parse_expression(&mut tokens.into_iter())
    }
}

#[derive(Debug)]
enum InBetween {
    Lexeme(Token),
    Parsed(ExpRef),
}

// Call only after initial parenthesis has been consumed
fn parse_inside_enclosure(
    tokens: &mut dyn Iterator<Item = Token>,
    enclosure: ExpEnclosure,
) -> ExpRef {
    let mut level = 1;
    let mut take_while = tokens.take_while(|token| {
        match enclosure {
            ExpEnclosure::Parenthesis => {
                match token {
                    Token::Symbol(Symbol::OpenParenthesis) => level += 1,
                    Token::Symbol(Symbol::CloseParenthesis) => level -= 1,
                    _ => {}
                };
            }
            ExpEnclosure::SquareBracket => {
                match token {
                    Token::Symbol(Symbol::OpenSquareBracket) => level += 1,
                    Token::Symbol(Symbol::CloseSquareBracket) => level -= 1,
                    _ => {}
                };
            }
        }

        level > 0
    });

    parse_expression(&mut take_while)
}

// Returns a Function Call expression with a null target field
fn parse_function_parameters(
    tokens: &mut dyn Iterator<Item = Token>,
    close_symbol: Symbol,
) -> Vec<String> {
    let mut parameters = Vec::new();
    while let Some(parameter) = tokens.next() {
        if let Token::Reference(name) = parameter {
            parameters.push(name);
        } else {
            if let Token::Symbol(symbol) = parameter {
                if symbol == close_symbol {
                    return Vec::new();
                }
            }
            panic!("Expected Function Parameter Name")
        }
        match tokens.next() {
            Some(Token::Symbol(Symbol::Comma)) => {}
            Some(Token::Symbol(symbol)) => {
                if symbol == close_symbol {
                    break;
                } else {
                    panic!("Expected Comma or Closing Parenthesis")
                }
            }
            Some(_) => panic!("Expected Comma or Closing Parenthesis"),
            None => panic!("Unexpected End of Line"),
        }
    }

    parameters
}

#[derive(Eq, PartialEq, Debug)]
enum ExpEnclosure {
    Parenthesis,
    SquareBracket,
}

fn parse_comma_list(
    tokens: &mut dyn Iterator<Item = Token>,
    enclosure: ExpEnclosure,
) -> Vec<ExpRef> {
    let mut next = true;
    let mut arguments = Vec::new();
    while next {
        let mut level = Vec::new();
        let mut take_while = tokens
            .take_while(|token| {
                match token {
                    Token::Symbol(Symbol::OpenParenthesis) => level.push(ExpEnclosure::Parenthesis),
                    Token::Symbol(Symbol::OpenSquareBracket) => {
                        level.push(ExpEnclosure::SquareBracket)
                    }
                    Token::Symbol(Symbol::CloseParenthesis) => match level.pop() {
                        Some(top) => assert_eq!(ExpEnclosure::Parenthesis, top),
                        None => {
                            assert_eq!(enclosure, ExpEnclosure::Parenthesis);
                            next = false;
                            return false;
                        }
                    },
                    Token::Symbol(Symbol::CloseSquareBracket) => match level.pop() {
                        Some(top) => assert_eq!(ExpEnclosure::SquareBracket, top),
                        None => {
                            assert_eq!(enclosure, ExpEnclosure::SquareBracket);
                            next = false;
                            return false;
                        }
                    },
                    Token::Symbol(Symbol::Comma) => {
                        if level.is_empty() {
                            return false;
                        }
                    }
                    _ => {}
                };

                true
            })
            .peekable();
        if take_while.peek().is_none() {
            assert!(!next);
            break;
        }
        let expr = parse_expression(&mut take_while);
        arguments.push(expr);
    }

    arguments
}

fn parser_pass<I>(
    progress: I,
    targets: Vec<(Token, Box<dyn Fn(ExpRef, ExpRef) -> Expression>)>,
) -> Vec<InBetween>
where
    I: IntoIterator<Item = InBetween>,
{
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = if let Lexeme(token) = next {
            if let Some((_, func)) = targets.iter().find(|(target, _)| target == &token) {
                let lhs_exp = match result.pop() {
                    Some(Parsed(exp_ref)) => exp_ref,
                    Some(Lexeme(token)) => panic!("{:?}", token),
                    None => panic!(),
                };
                let rhs_exp = match iter.next() {
                    Some(Parsed(exp_ref)) => exp_ref,
                    Some(Lexeme(token)) => panic!("{:?}", token),
                    None => panic!(),
                };
                let expr = func(lhs_exp, rhs_exp);
                Parsed(Box::new(Eval::Expression(expr)))
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
fn parser_pass_function_call<I>(progress: I) -> Vec<InBetween>
where
    I: IntoIterator<Item = InBetween>,
{
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = match next {
            Parsed(exp_ref) => match *exp_ref {
                Eval::Expression(Expression::FunctionCall { arguments, .. }) => {
                    let target = match result.pop() {
                        Some(Parsed(exp_ref)) => exp_ref,
                        other => panic!("{:?}", other),
                    };
                    let expr = Expression::FunctionCall { target, arguments };
                    Parsed(Box::new(Eval::Expression(expr)))
                }
                Eval::Expression(Expression::ListAccess { index, .. }) => {
                    let target = match result.pop() {
                        Some(Parsed(exp_ref)) => exp_ref,
                        other => panic!("{:?}", other),
                    };
                    let expr = Expression::ListAccess { target, index };
                    Parsed(Box::new(Eval::Expression(expr)))
                }
                others => Parsed(Box::new(others)),
            },
            Lexeme(Token::Symbol(Symbol::Period)) => {
                let lhs_exp = match result.pop() {
                    Some(Parsed(exp_ref)) => exp_ref,
                    other => panic!("{:?}", other),
                };
                let name = match iter.next() {
                    Some(Parsed(exp_ref)) => match *exp_ref {
                        Eval::Expression(Expression::Reference { target }) => target,
                        other => panic!("{:?}", other),
                    },
                    other => panic!("{:?}", other),
                };
                Parsed(Box::new(Eval::Expression(Expression::FieldAccess {
                    target: lhs_exp,
                    name,
                })))
            }
            others => others,
        };

        result.push(next_between);
    }

    result
}

fn parser_pass_unary<I>(
    progress: I,
    targets: Vec<(Token, Box<dyn Fn(ExpRef) -> Expression>, bool)>,
    // Bool represents if the operator should be ignored if it could be interpreted as a binary operator
    // (eg - could be minus or negative sign)
) -> Vec<InBetween>
where
    I: IntoIterator<Item = InBetween>,
{
    let mut result = Vec::new();
    let mut iter = progress.into_iter();
    while let Some(next) = iter.next() {
        let next_between = if let Lexeme(token) = next {
            if let Some((_, func, lhs_conditional)) =
                targets.iter().find(|(kind, _, _)| &token == kind)
            {
                let is_binary = match result.last() {
                    Some(Lexeme(_)) | None => false,
                    Some(Parsed(_)) => true,
                };
                if is_binary && *lhs_conditional {
                    Lexeme(token)
                } else {
                    let exp_ref = if let Some(Parsed(exp_ref)) = iter.next() {
                        exp_ref
                    } else {
                        panic!()
                    };
                    let expr = func(exp_ref);
                    Parsed(Box::new(Eval::Expression(expr)))
                }
            } else {
                Lexeme(token)
            }
        } else {
            next
        };

        result.push(next_between);
    }

    result
}

/// Parse an expression from tokens until a specified token is reached (consumes said token)
fn parse_expression(tokens: &mut dyn Iterator<Item = Token>) -> ExpRef {
    let mut between = Vec::new();
    while let Some(token) = tokens.next() {
        let next = match token {
            Token::Literal(literal) => Parsed(Box::new(Eval::Expression(Expression::Literal {
                value: literal,
            }))),
            Token::Reference(name) => Parsed(Box::new(Eval::Expression(Expression::Reference {
                target: name,
            }))),
            Token::Keyword(Keyword::This) => {
                Parsed(Box::new(Eval::Expression(Expression::ThisReference)))
            }
            Token::Keyword(Keyword::Self_) => {
                Parsed(Box::new(Eval::Expression(Expression::SelfReference)))
            }
            Token::Symbol(Symbol::OpenParenthesis) => {
                if let Some(Parsed(_)) = between.last() {
                    let args = parse_comma_list(tokens, ExpEnclosure::Parenthesis);
                    let expr = Expression::FunctionCall {
                        target: Box::new(Eval::Expression(Expression::Joiner {
                            expressions: vec![],
                        })),
                        arguments: args,
                    };
                    Parsed(Box::new(Eval::Expression(expr)))
                } else {
                    Parsed(parse_inside_enclosure(tokens, ExpEnclosure::Parenthesis))
                }
            }
            Token::Symbol(Symbol::OpenSquareBracket) => {
                if let Some(Parsed(_)) = between.last() {
                    let expr = parse_inside_enclosure(tokens, ExpEnclosure::SquareBracket);
                    let expr = Expression::ListAccess {
                        target: Box::new(Eval::Expression(Expression::Joiner {
                            expressions: vec![],
                        })),
                        index: expr,
                    };
                    Parsed(Box::new(Eval::Expression(expr)))
                } else {
                    let items = parse_comma_list(tokens, ExpEnclosure::SquareBracket);
                    let expr = Expression::ListDeclaration { values: items };
                    Parsed(Box::new(Eval::Expression(expr)))
                }
            }
            other_token => Lexeme(other_token),
        };
        between.push(next);
    }

    between = parser_pass_function_call(between);
    between = parser_pass_unary(
        between,
        vec![(
            Token::Symbol(Symbol::ExclamationPoint),
            Box::new(|target| Expression::Negate { operand: target }),
            false,
        )],
    );
    between = parser_pass(
        between,
        vec![(
            Token::Symbol(Symbol::DoubleStar),
            Box::new(|lhs, rhs| Expression::Exponent { lhs, rhs }),
        )],
    );
    between = parser_pass_unary(
        between,
        vec![(
            Token::Symbol(Symbol::Minus),
            Box::new(|target| Expression::Negate { operand: target }),
            true,
        )],
    );
    between = parser_pass(
        between,
        vec![
            (
                Token::Symbol(Symbol::Star),
                Box::new(|lhs, rhs| Expression::Multiply { lhs, rhs }),
            ),
            (
                Token::Symbol(Symbol::Slash),
                Box::new(|lhs, rhs| Expression::Divide { lhs, rhs }),
            ),
            (
                Token::Symbol(Symbol::DoubleSlash),
                Box::new(|lhs, rhs| Expression::DivideTruncate { lhs, rhs }),
            ),
            (
                Token::Symbol(Symbol::Percent),
                Box::new(|lhs, rhs| Expression::Modulus { lhs, rhs }),
            ),
        ],
    );
    between = parser_pass(
        between,
        vec![
            (
                Token::Symbol(Symbol::Plus),
                Box::new(|lhs, rhs| Expression::Addition { lhs, rhs }),
            ),
            (
                Token::Symbol(Symbol::Minus),
                Box::new(|lhs, rhs| Expression::Subtract { lhs, rhs }),
            ),
        ],
    );
    between = parser_pass(
        between,
        vec![(
            Token::Symbol(Symbol::And),
            Box::new(|lhs, rhs| Expression::And { lhs, rhs }),
        )],
    );
    between = parser_pass(
        between,
        vec![(
            Token::Symbol(Symbol::Or),
            Box::new(|lhs, rhs| Expression::Or { lhs, rhs }),
        )],
    );
    between = parser_pass(
        between,
        vec![
            (
                Token::Symbol(Symbol::DoubleEquals),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::Equal,
                }),
            ),
            (
                Token::Symbol(Symbol::Less),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::Less,
                }),
            ),
            (
                Token::Symbol(Symbol::LessEquals),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::LessEqual,
                }),
            ),
            (
                Token::Symbol(Symbol::Greater),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::Greater,
                }),
            ),
            (
                Token::Symbol(Symbol::GreaterEquals),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::GreaterEqual,
                }),
            ),
            (
                Token::Symbol(Symbol::NotEquals),
                Box::new(|lhs, rhs| Expression::Compare {
                    lhs,
                    rhs,
                    operation: Compare::NotEqual,
                }),
            ),
        ],
    );
    between = parser_pass(
        between,
        vec![(
            Token::Symbol(Symbol::Elvis),
            Box::new(|lhs, rhs| Expression::Elvis { lhs, rhs }),
        )],
    );
    between = parser_pass_unary(
        between,
        vec![
            (
                Token::Keyword(Keyword::Return),
                Box::new(|target| Expression::Return { value: target }),
                false,
            ),
            (
                Token::Keyword(Keyword::Yield),
                Box::new(|target| Expression::Yield { value: target }),
                false,
            ),
            (
                Token::Keyword(Keyword::Yeet),
                Box::new(|target| Expression::Yeet { value: target }),
                false,
            ),
        ],
    );
    let expr = match between.pop() {
        Some(Parsed(exp_ref)) => exp_ref,
        Some(Lexeme(token)) => panic!("{:?}", token),
        None => panic!(),
    };
    assert!(between.is_empty(), "{:?}", &between);
    expr
}
