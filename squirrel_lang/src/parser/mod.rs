mod ast;
mod lexer;

use std::iter::Peekable;

use ast::{ExprRef, StateRef};
use lexer::Token;
use logos::Span;
use serde::de::Unexpected;

use self::{
    ast::{Expr, Function, Literal, Statement},
    error::{ParseError, ParseResult},
    lexer::SpannedLexer,
};
use crate::context::{ContextSpan, ErrorContext};

fn parse<'s>(lexer: &mut SpannedLexer<'s>) -> Result<Vec<Statement>, ErrorContext> {
    parse_statements(lexer, None).map_err(|err| err.with_context(lexer))
}

fn parse_statements<'s>(
    tokens: &mut SpannedLexer<'s>,
    end_token: Option<Token>,
) -> ParseResult<Vec<Statement>> {
    let mut statements = Vec::new();
    loop {
        let next = tokens.peek();
        match (next, &end_token) {
            // Expected EOF
            (None, None) => break,
            // Expected End of Statement
            (Some(Ok((tok, _))), Some(end_token)) if tok == end_token => {
                tokens.next();
                break;
            }
            _ => {}
        }
        statements.push(parse_statement(tokens)?);
    }
    Ok(statements)
}

fn parse_statement<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<Statement> {
    let (initial_token, ctx) = if let Some(next) = tokens.next() {
        next?
    } else {
        return Err(ParseError::UnexpectedEof);
    };
    match initial_token {
        Token::LeftCurlyBrace => Ok(Statement::Block(parse_statements(
            tokens,
            Some(Token::RightCurlyBrace),
        )?)),
        Token::Newline | Token::Semicolon => Ok(Statement::Empty),
        other => Err(ParseError::UnexpectedToken(other, ctx)),
    }
}

fn parse_class<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<Statement> {
    let (mut first_token, mut ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
    let mut ident = None;
    if let Token::Identifier(name) = first_token {
        ident = Some(name);
        (first_token, ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
    }
    if first_token != Token::LeftCurlyBrace {
        return Err(ParseError::UnexpectedToken(first_token, ctx));
    }
    let body = parse_class_body(tokens)?;
    let expr = match ident {
        Some(ident) => Expr::NewSlot(ident, Box::new(body)),
        None => body,
    };
    Ok(Statement::Expr(expr))
}

fn parse_class_body<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<Expr> {
    let mut constructor = None;
    let mut members = Vec::new();
    loop {
        let next = tokens.peek().ok_or(error::ParseError::UnexpectedEof)?;
        match next {
            Ok((Token::RightCurlyBrace, _)) => {
                tokens.next();
                break;
            },
            Ok((Token::Constructor, _)) => {
                tokens.next();
                constructor = Some(parse_function_args_body(tokens)?);
            },
            Ok((Token::Function, _)) => {
                let (name, func) = parse_function(tokens)?;
                members.push((Expr::Literal(Literal::String(name)), Expr::FunctionDef(func)));
            },
            other => {
                members.push(parse_table_slot(tokens)?);
            }
        }
    }
    Ok(Expr::ClassDef {
        constructor, members
    })
}

fn parse_table_slot<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<(Expr, Expr)> {
    let (init_token, ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
    match init_token {
        Token::Identifier(name) => {
            let (next, ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
            if next != Token::Assign {
                return Err(ParseError::UnexpectedToken(next, ctx));
            }
            let value = parse_expr(tokens)?;
            Ok((Expr::Literal(Literal::String(name)), value))
        },
        Token::LeftSquareBracket => {
            let key = parse_expr(tokens)?;
            let (next, ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
            if next != Token::Assign {
                return Err(ParseError::UnexpectedToken(next, ctx));
            }
            let value = parse_expr(tokens)?;
            Ok((key, value))
        },
        other => Err(ParseError::UnexpectedToken(other, ctx)),
    }
}

fn parse_expr<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<Expr> {
    todo!()
}

fn parse_function<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<(String, Function)> {
    let (name_tok, ctx) = tokens.next().ok_or(ParseError::UnexpectedEof)??;
    let name = match name_tok {
        Token::Identifier(name) => name,
        other => return Err(ParseError::UnexpectedToken(other, ctx)),
    };
    Ok((name, parse_function_args_body(tokens)?))
}

fn parse_function_args_body<'s>(tokens: &mut SpannedLexer<'s>) -> ParseResult<Function> {
    todo!()
}

mod error {
    use logos::Span;

    use crate::context::{ContextSpan, ErrorContext};

    use super::lexer::{SpannedLexer, Token};

    pub type ParseResult<T> = Result<T, ParseError>;
    pub enum ParseError {
        UnexpectedToken(Token, ContextSpan),
        UnexpectedEof,
        ErrorContext(ErrorContext),
    }

    impl From<ErrorContext> for ParseError {
        fn from(ctx: ErrorContext) -> Self {
            Self::ErrorContext(ctx)
        }
    }

    impl ParseError {
        pub fn with_context(self, ctx: &SpannedLexer) -> ErrorContext {
            match self {
                ParseError::UnexpectedToken(tok, span) => ErrorContext::new(
                    ctx.get_file_name().to_string(),
                    span.end_line,
                    ctx.get_source(),
                    span.into(),
                    format!("Unexpected token {:?}", tok),
                ),
                ParseError::UnexpectedEof => ErrorContext::new(
                    ctx.get_file_name().to_string(),
                    ctx.current_line(),
                    ctx.get_source(),
                    Span {
                        start: ctx.current_offset(),
                        end: ctx.current_offset(),
                    },
                    format!("Unexpected eof"),
                ),
                ParseError::ErrorContext(err) => err,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn squirrel_sample_test() {
        for file in
            fs::read_dir("../squirrel/samples/").expect("Unable to find squirrel samples directory")
        {
            let file = file.expect("Unable to read squirrel samples directory");
            let path = file.path();
            let contents = fs::read_to_string(&path).unwrap();
            let mut spanned_lexer =
                SpannedLexer::new(&contents, path.to_string_lossy().to_string());
            let ast= parse(&mut spanned_lexer).unwrap_or_else(|err| panic!("{:#}", err));

            // TODO: Do something with ast
        }
    }
}
