
use nom::{
    branch::alt, bytes::complete::{is_not, tag, take, take_until}, character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    }, combinator::{all_consuming, map_res, opt, recognize, value}, error::{Error, ErrorKind, ParseError}, multi::{fold_many1, many0_count, many1, separated_list0}, sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, Compare, CompareResult, Err, IResult, InputIter, InputLength, InputTake, Needed, Slice
};

use crate::ir::{
    builders::IRBuilder, structures::*, types::Type, values
};

use super::lexer::Lexer;
use super::token::{Token, Tokens};


fn token<'a, Input, Error: ParseError<Input>>(
    t: Token<'a>
) -> impl Fn(Input) -> IResult<Input, Input, Error> + 'a
where
    Input: InputTake + Compare<Token<'a>>
{
    move | i: Input | {
        let token_len = t.input_len();
        let t = t.clone();
        let res: IResult<_, _, Error> = match i.compare(t) {
            CompareResult::Ok => Ok(i.take_split(token_len)),
            _ => {
                let e: ErrorKind = ErrorKind::Tag;
                Err(Err::Error(Error::from_error_kind(i, e)))
            }
        };
        res
    }
}

fn identifier(input: Tokens) -> IResult<Tokens, &str> {
    let (input, tk) = take(1usize)(input)?;
    match tk.iter_elements().next().unwrap() {
        Token::TkIdent(id) => Ok((input, id)),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn i64_literal(input: Tokens) -> IResult<Tokens, i64> {
    let (input, tk) = take(1usize)(input)?;
    match tk.iter_elements().next().unwrap() {
        Token::LtInt64(value) => Ok((input, value.clone())),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn i1_literal(input: Tokens) -> IResult<Tokens, i8> {
    let (input, tk) = take(1usize)(input)?;
    match tk.iter_elements().next().unwrap() {
        Token::LtInt1(value) => Ok((input, value.clone())),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}




fn parse_base_type(input: Tokens) -> IResult<Tokens, Type> {
    alt((
        value(Type::get_i64(), token(Token::TyInt64)),
        value(Type::get_i1(), token(Token::TyInt1)),
        value(Type::get_unit(), pair(
            token(Token::LParen), token(Token::RParen)))
    ))(input)
}

fn parse_pointer_type(input: Tokens) -> IResult<Tokens, Type> {
    let (input, base_ty) = parse_base_type(input)?;
    fold_many1(
        token(Token::Asterisk),
        move | | base_ty.clone(),
        | ty: Type, _| Type::get_pointer(ty)
    )(input)
}

fn parse_function_type(input: Tokens) -> IResult<Tokens, Type> {
    /* format: fn(param_ty1, param_ty2, ...) -> return_ty */
    let (input, params) = preceded(
        token(Token::KwFn),
        delimited(
            token(Token::LParen),
            separated_list0(
                token(Token::Comma),
                parse_type,
            ),
            token(Token::RParen)
        )
    )(input)?;

    let (input, ret) = preceded(
        token(Token::Arrow),
        parse_type
    )(input)?;
    Ok((input, Type::get_function(params, ret)))
}

fn parse_type(input: Tokens) -> IResult<Tokens, Type> {
    alt((
        parse_pointer_type,
        parse_base_type,
        /*  opaque pointer type */
        value(Type::get_opaque_pointer(), token(Token::TyPtr)),
        parse_function_type,
    ))(input)
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
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
    ))(input)
}

fn parse_symbol(input: Tokens) -> IResult<Tokens, (&str, Option<Type>)> {
    pair(
        identifier,
        opt(preceded(token(Token::Colon), parse_type))
    )(input)
}

pub struct Parser {
    lexer: Lexer
}

impl Parser {
    pub fn new() -> Parser {
        Parser { lexer: Lexer::new() }
    }


}



#[cfg(test)]
mod test {
    use super::*;
    use super::super::lexer::Lexer;


    macro_rules! test_parser {
        ($parser: ident, $input: expr, $result: expr) => {
            let (res_input, tokens) = Lexer::lex($input).unwrap();
            assert!(res_input.is_empty(), "lexer leaves out unlexed tokens");
            let (_, output) = $parser(Tokens::new(&tokens)).unwrap();
            assert_eq!(output, $result);          
        };
    }

    #[test]
    fn test_ident() {
        test_parser!(parse_symbol, "%a.very.long.identifier", ("a.very.long.identifier", None));
        test_parser!(parse_symbol, "@12", ("12", None));
        test_parser!(parse_symbol, "%res: i64", ("res", Some(Type::get_i64())));
        test_parser!(parse_symbol, "%implicit_symbol", ("implicit_symbol", None));
        test_parser!(parse_symbol, "#DivisionByZero: fn(i64, i64) -> ()", 
                    ("DivisionByZero", Some(Type::get_function(vec![Type::get_i64(), Type::get_i64()], Type::get_unit()))));
    }

    #[test]
    fn test_type() {
        test_parser!(parse_type, "i64", Type::get_i64());
        test_parser!(parse_type, "i1", Type::get_i1());
        test_parser!(parse_type, "()", Type::get_unit());
        test_parser!(parse_type, "i64*", Type::get_pointer(Type::get_i64()));
        test_parser!(parse_type, "i64**", Type::get_pointer(Type::get_pointer(Type::get_i64())));
        test_parser!(parse_type, "fn() -> ()", 
                Type::get_function(vec![], Type::get_unit()));
        test_parser!(parse_type, "fn(i64, i64) -> ()", 
                Type::get_function(vec![Type::get_i64(), Type::get_i64()], Type::get_unit()));
        test_parser!(parse_type, "fn((), i64) -> i64*",
                Type::get_function(vec![Type::get_unit(), Type::get_i64()], Type::get_pointer(Type::get_i64())));
        test_parser!(parse_type, "fn(fn(i64) -> i64 ) -> ()",
                Type::get_function(vec![Type::get_function(vec![Type::get_i64()], Type::get_i64())], Type::get_unit()));
    }
}


