use chumsky::prelude::*;
use chumsky::text::*;
use chumsky::input::*;
use chumsky::util::*;
use chumsky::error::*;
use chumsky::extra::*;

use crate::frontend::{
    token::Token,
    ParserError, Spanned
};

pub fn accipit_named_ident<'a, I: ValueInput<'a> + StrInput<'a, C>, C: Char, E: ParserExtra<'a, I>>(
) -> impl Parser<'a, I, &'a C::Str, E> + Copy + Clone {
    any()
        // Use try_map over filter to get a better error on failure
        .try_map(|c: C, span| {
            if c.to_char().is_ascii_alphabetic() || c.to_char() == '_' || c.to_char() == '.' || c.to_char() == '-'{
                Ok(c)
            } else {
                Err(Error::expected_found([], Some(MaybeRef::Val(c)), span))
            }
        })
        .then(
            any()
                // This error never appears due to `repeated` so can use `filter`
                .filter(|c: &C| c.to_char().is_ascii_alphanumeric() || c.to_char() == '_' || c.to_char() == '.' || c.to_char() == '.')
                .repeated(),
        )
        .to_slice()
}


pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<Spanned<Token<'a>>>, ParserError<'a, char>> {
    let binop = choice([
        keyword("add").to(Token::TkAdd),
        keyword("sub").to(Token::TkSub),
        keyword("mul").to(Token::TkMul),
        keyword("div").to(Token::TkDiv),
        keyword("rem").to(Token::TkRem),
        keyword("and").to(Token::TkAnd),
        keyword("or").to(Token::TkOr),
        keyword("xor").to(Token::TkXor),
        keyword("lt").to(Token::TkLt),
        keyword("gt").to(Token::TkGt),
        keyword("le").to(Token::TkLe),
        keyword("ge").to(Token::TkGe),
        keyword("eq").to(Token::TkEq),
        keyword("ne").to(Token::TkNe)
    ]);

    let instruction_opcode = choice([
        // instruction opcode keywords
        keyword("offset").to(Token::TkOffset),
        keyword("alloca").to(Token::TkAlloca),
        keyword("load").to(Token::TkLoad),
        keyword("store").to(Token::TkStore),
        keyword("call").to(Token::TkFnCall)
    ]).or(binop);

    let terminator_opcode = choice([
        keyword("jmp").to(Token::TkJmp),
        keyword("br").to(Token::TKBranch),
        keyword("ret").to(Token::TKRet)
    ]);

    let opcode = instruction_opcode.or(terminator_opcode);
    
    let delimiter = 
        just::<char, &'a str, ParserError<'a, char>>('-').then_ignore(just('>')).to(Token::Arrow)
    .or(choice([
        just::<char, &'a str, ParserError<'a, char>>('(').to(Token::LParen),
        just(')').to(Token::RParen),
        just('[').to(Token::LBracket),
        just(']').to(Token::RBracket),
        just('{').to(Token::LBrace),
        just('}').to(Token::RBrace),
        just('=').to(Token::Equal),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just(';').to(Token::SemiColon),
        just('<').to(Token::Less),
        just('*').to(Token::Asterisk)
    ]));

    let keywords = choice([
        keyword("fn").to(Token::KwFn),
        keyword("let").to(Token::KwLet),
        keyword("label").to(Token::KwLabel),
    ]);

    let primitive_type = choice([
        keyword("i64").to(Token::TyInt64),
        keyword("i1").to(Token::TyInt1),
        keyword("ptr").to(Token::TyPtr),
    ]);

    let int_literal = int::<_, _, ParserError<'a, char>>(10)
        .from_str::<i64>()
        .unwrapped()
        .map(Token::LtInt64);

    let literal = int_literal.or(choice([
        keyword("true").to(Token::LtInt1(true)),
        keyword("false").to(Token::LtInt1(false)),
        keyword("none").to(Token::LtNone),
        keyword("null").to(Token::LtNull)
    ]));

    let ident = one_of::<_, _, ParserError<'a, char>>("%@#")
        .ignore_then(
            accipit_named_ident::<&'a str, char, ParserError<'a, char>>()
            .or(digits(10).to_slice())
        )
        .map(Token::TkIdent);
    
    let single_comment = just::<_, &str, ParserError<'a, char>>("//")
        .ignore_then(none_of("\n").repeated())
        .then_ignore(just("\n"))
        .padded();

    let multi_comment = just::<_, &str, ParserError<'a, char>>("/*")
        .ignore_then(none_of("*/").repeated())
        .then_ignore(just("*/"))
        .padded();

    let comment = single_comment.or(multi_comment);
    
    let recovery =
        // filter current unknown token
        none_of::<&str, &str, ParserError<'a, char>>(" \t\n\r").repeated();

    // tokens
    let token = choice((
        keywords,
        literal,
        ident,
        primitive_type,
        delimiter,
        opcode,
    ));

    token
        .map_with(| tok, extra| Spanned(tok, extra.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(recovery.ignored(), end()))
        .map(| tok | { println!("{:?}", tok);  tok})
        .repeated()
        .collect::<Vec<Spanned<Token<'a>>>>()
}