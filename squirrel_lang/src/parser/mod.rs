mod lexer;
mod context;
mod ast;

use lexer::Token;
use logos::Span;
use ast::{ExprRef, StateRef};

fn parse(tokens: impl Iterator<Item = (Token, Span)>) {

}

fn parse_statement<L: Iterator<Item = (Token, Span)>>(tokens: &L) -> StateRef {
    todo!()
}