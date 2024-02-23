
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    },
    combinator::{all_consuming, map_res, recognize, value, opt},
    multi::{fold_many1, many0_count, many1, separated_list0}, 
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, 
    Err, IResult
};

use crate::ir::{
    types::Type, 
    values,
    structures::*
};


fn parse_usize_imm(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse::<usize>)(input)
}

fn parse_i64_constant(input: &str) -> IResult<&str, Value> {
    let (input, value) = map_res(
        recognize(pair(opt(tag("-")), digit1)),
        str::parse::<i64>
    )(input)?;
    Ok((input, values::ConstantInt::new_value(value)))
}

fn parse_i8_cosntant(input: &str) -> IResult<&str, Value> {
    alt((
        value(values::ConstantInt::new_true_value(), tag("true")),
        value(values::ConstantInt::new_false_value(), tag("false"))
    ))(input)
}


fn parse_i64_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_i64(), tag("i64"))(input)
}

fn parse_i1_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_i1(), tag("i1"))(input)
} 

fn parse_unit_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_unit(), tag("()"))(input)
}

fn parse_base_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_i64_type,
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


fn parse_function_type(input: &str) -> IResult<&str, Type> {
    let (input, params) = preceded(
        tag("fn"), 
        delimited(
            tag("("), 
            separated_list0(tag(","), 
                delimited(
                    multispace0, 
                    parse_type, 
                    multispace0
                )
            ),
            tag(")")
        )    
    )(input)?;
    let (input, ret) = preceded(
        delimited(
            multispace0,
            tag("->"),
            multispace0
        ), 
        parse_type
    )(input)?;
    Ok((input, Type::get_function(params, ret)))
}


fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_pointer_type,
        parse_base_type,
        parse_function_type
    ))(input)
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

fn parse_symbol(input: &str) -> IResult<&str, (&str, Option<Type>)> {
    let (input, ident) = alt((parse_vident, parse_pident, parse_gident))(input)?;
    let (input, ty) = opt(preceded(
        delimited(
            multispace0,
            tag(":"), 
            multispace0
        ),
        parse_type
    ))(input)?;
    Ok((input, (ident, ty)))

}

fn parse_binop(input: &str) -> IResult<&str, values::BinaryOp> {
    alt((
        value(values::BinaryOp::Add, tag("add")),
        value(values::BinaryOp::Sub, tag("sub")),
        value(values::BinaryOp::Mul, tag("mul")),
        value(values::BinaryOp::Div, tag("div")),
        value(values::BinaryOp::Rem, tag("rem")),
        value(values::BinaryOp::And, tag("and")),
        value(values::BinaryOp::Or, tag("or")),
        value(values::BinaryOp::Xor, tag("xor")),
        value(values::BinaryOp::Lt, tag("lt")),
        value(values::BinaryOp::Gt, tag("gt")),
        value(values::BinaryOp::Le, tag("le")),
        value(values::BinaryOp::Ge, tag("ge")),
        value(values::BinaryOp::Eq, tag("eq")),
        value(values::BinaryOp::Ne, tag("ne")),
    ))(input)
}


#[test]
fn test_literal() {
    assert_eq!(parse_usize_imm("123456789"), Ok(("", 123456789usize)));
}

#[test]
fn test_ident() {
    assert_eq!(parse_ident("a.very.long.identifier"), Ok(("", "a.very.long.identifier")));
    assert_eq!(parse_ident("12"), Ok(("", "12")));
    assert_eq!(parse_symbol("%res: i64"), Ok(("", ("res", Some(Type::get_i64())))));
    assert_eq!(parse_symbol("%implicit_symbol "), Ok((" ", ("implicit_symbol", None))));
}

#[test]
fn test_type() {
    assert_eq!(parse_type("i64"), Ok(("", Type::get_i64())));
    assert_eq!(parse_type("i1"), Ok(("", Type::get_i1())));
    assert_eq!(parse_type("()"), Ok(("", Type::get_unit())));
    assert_eq!(parse_type("i64*"), Ok(("", Type::get_pointer(Type::get_i64()))));
    assert_eq!(parse_type("i64**"), Ok(("", Type::get_pointer(Type::get_pointer(Type::get_i64())))));
    assert_eq!(parse_type("fn() -> ()"), 
            Ok(("", Type::get_function(vec![], Type::get_unit()))));
    assert_eq!(parse_type("fn(i64, i64) -> ()"), 
            Ok(("", Type::get_function(vec![Type::get_i64(), Type::get_i64()], Type::get_unit()))));
    assert_eq!(parse_type("fn((), i64) -> i64*"),
            Ok(("", Type::get_function(vec![Type::get_unit(), Type::get_i64()], Type::get_pointer(Type::get_i64())))));
    assert_eq!(parse_type("fn(fn(i64) -> i64 ) -> ()"),
            Ok(("", Type::get_function(vec![Type::get_function(vec![Type::get_i64()], Type::get_i64())], Type::get_unit()))));
}