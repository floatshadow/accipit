
use nom::{
    branch::alt, bytes::complete::{is_not, tag, take_until}, character::complete::{
        char, alpha0, alpha1, alphanumeric0, alphanumeric1, digit0, digit1, multispace0, multispace1
    }, combinator::{map_res, recognize, value}, multi::{many0_count, many1_count}, sequence::{pair, tuple}, IResult
};

use crate::ir::{
    types::Type, 
    values::*
};




fn parse_i32_type(input: &str) -> IResult<&str, Type> {
    value(Type::get_i32(), tag("i32"))(input)
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

#[test]
fn ident() {
    assert_eq!(parse_ident("a.very.long.identifier"), Ok(("", "a.very.long.identifier")));
}