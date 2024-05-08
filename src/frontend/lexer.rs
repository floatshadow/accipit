
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    },
    combinator::{all_consuming, cut, map_res, opt, recognize, value},
    multi::{fold_many1, many0, many0_count, many1, separated_list0}, 
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, 
    Err
};

use crate::ir::builders::IRBuilder;

use super::token::Token;

pub type IResult<I, O, E=nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

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

fn lex_i32_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    let (input, value) = map_res(
        recognize(pair(opt(tag("-")), digit1)),
        str::parse::<i32>
    )(input)?;
    Ok((input, Token::LtInt32(value)))
}

fn lex_i1_literal(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    alt((
        value(Token::LtInt1(true), tag("true")),
        value(Token::LtInt1(false), tag("false"))
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
        lex_i32_literal,
        lex_i1_literal,
        lex_none_literal,
        lex_nullptr_literal
    ))(input)
}

fn lex_keyword(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::KwFn, tag("fn")),
            value(Token::KwLet, tag("let")),
            value(Token::KwLabel, tag("label")),
            value(Token::KwRegion, tag("region"))
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
            value(Token::Colon,     tag(":")),
            value(Token::SemiColon, tag(";")),
            value(Token::Less,      tag("<")),
            value(Token::Asterisk,  tag("*")),          
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

fn lex_offset_operator(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        value(Token::TkOffset, tag("offset"))
    )(input)
}

fn lex_memory_operator(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::TkAlloca, tag("alloca")),
            value(Token::TkLoad,   tag("load")),
            value(Token::TkStore,  tag("store"))
        ))
    )(input)
}

fn lex_terminator_operator(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        alt((
            value(Token::TkJmp,    tag("jmp")),
            value(Token::TKBranch, tag("br")),
            value(Token::TKRet,    tag("ret")),
            value(Token::KwRegion, tag("region"))
        ))
    )(input)
}

fn lex_function_cal_operator(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment,
        value(Token::TkFnCall, tag("call"))
    )(input)
}

fn lex_identifier(input: &str) -> IResult<&str, Token> {
    let (input, _) = filter_whitespace_and_comment(input)?;
    let (input, slice) = preceded(
        alt((tag("%"), tag("#"), tag("@"))),
        alt((
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
    )))(input)?;
    Ok((input, Token::TkIdent(slice)))
}

fn lex_primitive_type(input: &str) -> IResult<&str, Token> {
    preceded(filter_whitespace_and_comment, alt((
        value(Token::TyInt32,tag("i32")),
        value(Token::TyInt1, tag("i1")),
        // value(Token::TyUnit, tag("()")),
        value(Token::TyPtr,  tag("ptr")),
    )))(input)
}

#[derive(Debug, Clone)]
pub struct Lexer;

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {}
    }

    pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
        all_consuming(
            many1(terminated(alt((
                // `let` `le` has name collision.
                lex_keyword,
                lex_literal,
                lex_identifier,
                lex_primitive_type,
                lex_delimiter,
                lex_binary_operator,
                lex_offset_operator,
                lex_memory_operator,
                lex_function_cal_operator,
                lex_terminator_operator,
            )),
            filter_whitespace_and_comment,
        )))(input)
    }
}