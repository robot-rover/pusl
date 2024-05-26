mod ast;
mod lexer;

use std::iter::Peekable;

use ast::{ExprRef, StateRef};
use lexer::Token;
use logos::Span;
use serde::de::Unexpected;

use self::{
    ast::Statement,
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
