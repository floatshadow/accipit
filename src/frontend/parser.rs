
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    },
    combinator::{all_consuming, map_res, recognize, value, peek},
    multi::{fold_many1, many0_count, many1, separated_list1}, 
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, 
    Err, IResult
};

use crate::ir::{
    types::Type, 
    values::*
};




fn parse_i32_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_i32(), tag("i32"))(input)
}

fn parse_i1_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_i1(), tag("i1"))(input)
} 

fn parse_unit_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_unit(), tag("()"))(input)
}

fn parse_base_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_i32_type,
        parse_i1_type,
        parse_unit_type
    ))(input)
}

fn parse_pointer_type(input: &str) -> IResult<&str, Type> {
    let (input, base_type) = parse_base_type(input)?;
    let res = fold_many1(
        char('*'), 
        || base_type.clone(),
        |ty: Type, _ | Type::get_pointer(ty)
    )(input);
    res
}


fn parse_function_ty_curry_ty(input: &str) -> IResult<&str, Type> {
    delimited(
        multispace0,
        alt((
            preceded(peek(parse_pointer_type), parse_pointer_type),
            parse_i32_type,
            parse_i1_type,
            preceded(peek(parse_unit_type), parse_unit_type),
            delimited(
            tag("("), 
            parse_function_type,
            tag(")")
            ),
        )),
        multispace0
    )(input)
}

fn parse_function_type(input: &str) -> IResult<&str, Type> {
    let mut params_parser = many1(
        terminated(
            parse_function_ty_curry_ty,
            tag("->")
        )
    );

    let (input, params) = params_parser(input)?;
    let (input, ret) = parse_function_ty_curry_ty(input)?;
    Ok((input, Type::get_function(params, ret)))
}


fn filter_comment(input: &str) -> IResult<&str, ()> {
    alt((
        /* C EOL style */
        value((), pair(tag("//"), is_not("\n\r"))),
        /* C++ multiline style */
        value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))
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

fn parse_vident(input: &str) -> IResult<&str, &str> {
    preceded(tag("%"), parse_ident)(input)
}

fn parse_pident(input: &str) -> IResult<&str, &str> {
    preceded(tag("#"), parse_pident)(input)
}

fn parse_gident(input: &str) -> IResult<&str, &str> {
    preceded(tag("@"), parse_ident)(input)
}



#[test]
fn test_ident() {
    assert_eq!(parse_ident("a.very.long.identifier"), Ok(("", "a.very.long.identifier")));
    assert_eq!(parse_ident("12"), Ok(("", "12")));
}

#[test]
fn test_type() {
    assert_eq!(parse_function_type("i32 -> i32 -> ()"), 
            Ok(("", Type::get_function(vec![Type::get_i32(), Type::get_i32()], Type::get_unit()))));
    assert_eq!(parse_function_type("() -> i32 -> i32*"),
            Ok(("", Type::get_function(vec![Type::get_unit(), Type::get_i32()], Type::get_pointer(Type::get_i32())))));
    assert_eq!(parse_function_type("(i32 -> i32) -> ()"),
            Ok(("", Type::get_function(vec![Type::get_function(vec![Type::get_i32()], Type::get_i32())], Type::get_unit()))));
}