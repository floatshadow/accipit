
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    },
    combinator::{all_consuming, map_res, opt, recognize, value},
    multi::{fold_many1, many0, many0_count, many1, separated_list0}, 
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, 
    Err, IResult
};

use crate::ir::builders::IRBuilder;

use super::token::Token;

fn filter_comment(input: &str) -> IResult<&str, ()> {
    alt((
        /* C EOL style */
        value((), pair(tag("//"), is_not("\n\r"))),
        /* C++ multiline style */
        value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))
    ))(input)
}

fn filter_whitespace_and_comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0_count(alt((
            value((), multispace1),
            filter_comment
        ))
    ))(input)
}

fn lex_i64_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    let (input, value) = map_res(
        recognize(pair(opt(tag("-")), digit1)),
        str::parse::<i64>
    )(input)?;
    Ok((input, Token::LtInt64(value)))
}

fn lex_i1_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    alt((
        value(Token::LtTkInt1(1), tag("true")),
        value(Token::LtTkInt1(0), tag("false"))
    ))(input)
}

fn lex_none_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    value(Token::LtNone, tag("none"))(input)
}

fn lex_nullptr_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    value(Token::LtNull, tag("null"))(input)
}

fn lex_literal(input: &str) -> IResult<&str, Token> {
    alt((
        lex_i64_literal,
        lex_i1_literal,
        lex_none_literal,
        lex_nullptr_literal
    ))(input)
}

fn lex_keyword(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::KwFn, tag("fn")),
            value(Token::KwLet, tag("let"))
    )))(input)
}

fn lex_delimiter(input: &str) -> IResult<&str, Token> { 
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::LParen,    tag("(")),
            value(Token::RParen,    tag(")")),
            value(Token::LBracket,  tag("[")),
            value(Token::RBracket,  tag("]")),
            value(Token::LBrace,    tag("{")),
            value(Token::RBrace,    tag("}")),
            value(Token::Arrow,     tag("->")),
            value(Token::Equal,     tag("=")),
            value(Token::Comma,     tag(",")),
            value(Token::SemiColon, tag(";")),
            value(Token::Less,      tag("<")),
            value(Token::Asterisk,  tag("*")),          
    )))(input)
}

fn lex_prefix(input: &str) -> IResult<&str, Token> { 
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::PxAt,      tag("@")),
            value(Token::PxPercent, tag("%")),
            value(Token::PxSharp,   tag("#")), 
    )))(input)
}


fn lex_binary_operator(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::TkAdd,    tag("add")),
            value(Token::TkSub,    tag("sub")),
            value(Token::TkMul,    tag("mul")),
            value(Token::TkDiv,    tag("div")),
            value(Token::TkRem,    tag("rem")),
            value(Token::TkAnd,    tag("and")),
            value(Token::TkOr,     tag("or")),
            value(Token::TkXor,    tag("xor")),
            value(Token::TkLt,     tag("lt")),
            value(Token::TkGt,     tag("gt")),
            value(Token::TkLe,     tag("le")),
            value(Token::TkGe,     tag("ge")),
            value(Token::TkEq,     tag("eq")),
            value(Token::TkNe,     tag("ne")),     
    )))(input)
}


fn lex_identifier(input: &str) -> IResult<&str, Token> {
    let (input, slice) = alt((
        /* named identifier */
        recognize(
            pair(
                alt((alpha1, tag("_"), tag("."), tag("-"))),
                many0_count(alt((alphanumeric1, tag("_"), tag("."), tag("-"))))
            )
        ),
        /* anonymous identifer */
        recognize(
            digit1
        )
    ))(input)?;
    Ok((input, Token::TkIdent(slice)))
}

fn lex_primitive_type(input: &str) -> IResult<&str, Token> {
    alt((
        value(Token::TyInt64, tag("i64")),
        value(Token::TyInt1, tag("i1")),
        value(Token::TyUnit, tag("()"))
    ))(input)
}

#[derive(Debug, Clone)]
pub struct Lexer;