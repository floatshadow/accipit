pub mod parser;
pub mod lexer;
pub mod token;


use chumsky::prelude::{extra, Rich, SimpleSpan};

pub type Span = SimpleSpan<usize>;
pub type ParserError<'a, T> = extra::Err<Rich<'a, T, Span>>;
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T>(T, Span);

pub mod new_lexer;