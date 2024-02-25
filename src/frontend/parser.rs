use std::cell::RefCell;
use std::rc::Rc;
use nom::{
    branch::alt, bytes::complete::{is_not, tag, take, take_until}, character::complete::{
        alpha0, alpha1, 
        alphanumeric0, alphanumeric1, 
        char, digit0, digit1, 
        multispace0, multispace1
    }, combinator::{all_consuming, map, map_res, opt, peek, recognize, value}, error::{Error, ErrorKind, ParseError}, multi::{fold_many1, many0, many0_count, many1, many1_count, separated_list0}, sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, Compare, CompareResult, Err, IResult, InputIter, InputLength, InputTake, Needed, Slice
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

fn label(input: Tokens) -> IResult<Tokens, &str> {
    let (input, tk) = take(1usize)(input)?;
    match tk.iter_elements().next().unwrap() {
        Token::TkLabel(id) => Ok((input, id)),
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

fn parse_binop(input: Tokens) -> IResult<Tokens, values::BinaryOp> {
    alt((
        value(values::BinaryOp::Add,    token(Token::TkAdd)),
        value(values::BinaryOp::Sub,    token(Token::TkSub)),
        value(values::BinaryOp::Mul,    token(Token::TkMul)),
        value(values::BinaryOp::Div,    token(Token::TkDiv)),
        value(values::BinaryOp::Rem,    token(Token::TkRem)),
        value(values::BinaryOp::And,    token(Token::TkAnd)),
        value(values::BinaryOp::Or,     token(Token::TkOr)),
        value(values::BinaryOp::Xor,    token(Token::TkXor)),
        value(values::BinaryOp::Lt,     token(Token::TkLt)),
        value(values::BinaryOp::Gt,     token(Token::TkGt)),
        value(values::BinaryOp::Le,     token(Token::TkLe)),
        value(values::BinaryOp::Ge,     token(Token::TkGe)),
        value(values::BinaryOp::Eq,     token(Token::TkEq)),
        value(values::BinaryOp::Ne,     token(Token::TkNe)),
    ))(input)
}

#[derive(Debug, Clone)]
pub struct Parser;

impl<'a, 'b: 'a> Parser {
    pub fn new() -> Parser {
        Parser { }
    }

    fn parse_binary_expr(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ValueRef> {
        let (input,(name, anno_ty)) =
            delimited(token(Token::KwLet), parse_symbol, token(Token::Equal))(input)?;
        let (input, (op, ((lhs, _), (rhs, _)))) = tuple((
            parse_binop,
            separated_pair(parse_symbol, token(Token::Comma), parse_symbol)
        ))(input)?;

        let lhs_ref = builder.borrow().get_value_ref(lhs);
        let rhs_ref = builder.borrow().get_value_ref(rhs);

        Ok((input, builder.borrow_mut().emit_numeric_binary_expr(
            op,
            Some(String::from(name)),
            lhs_ref,
            rhs_ref,
            anno_ty
        )))
    }

    fn parse_instruction(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>,
    ) -> IResult<Tokens<'a>, ValueRef> {
        alt((
            | input: Tokens<'a> | Parser::parse_binary_expr(input, builder.clone()),
        ))(input)
    }

    fn parse_terminator_jump(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        let (input, dest) = preceded(
            token(Token::TkJmp),
            preceded(token(Token::KwLabel), identifier)
        )(input)?;
        let dest_ref = builder.borrow().get_block_ref(dest);
        builder.borrow_mut().fixup_terminator_jump(dest_ref);
        Ok((input, ()))
    }

    fn parse_terminator_branch(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        let (input, ((cond, _), (true_label, false_label))) = preceded(
            token(Token::TKBranch),
            tuple((
                parse_symbol,
                separated_pair(
                    preceded(token(Token::KwLabel),   identifier),
                    token(Token::Comma),
                    preceded(token(Token::KwLabel),   identifier)
                )
            ))
        )(input)?;
        let cond_ref = builder.borrow().get_value_ref(cond);
        let true_ref = builder.borrow().get_block_ref(true_label);
        let false_ref = builder.borrow().get_block_ref(false_label);
        builder.borrow_mut().fixup_terminator_branch(cond_ref, true_ref, false_ref);
        Ok((input, ()))
    }

    fn parse_terminator_return(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        let (input, (ret, _)) = preceded(token(Token::TKRet), parse_symbol)(input)?;
        let ret_ref = builder
            .borrow()
            .get_value_ref(ret);
        builder
            .borrow_mut()
            .fixup_terminator_return(ret_ref);
        Ok((input, ()))
    }

    fn parse_terminator(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        alt((
            | input: Tokens<'a> | Parser::parse_terminator_jump(input, builder.clone()),
            | input: Tokens<'a> | Parser::parse_terminator_branch(input, builder.clone()),
            | input: Tokens<'a> | Parser::parse_terminator_return(input, builder.clone())
        ))(input)
    }

    fn parse_basic_block(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, BlockRef> {
        let (input, label) = label(input)?;
        let handler = builder
            .borrow_mut()
            .emit_basic_block(Some(String::from(label)));
        builder
            .borrow_mut()
            .set_insert_point(handler);
        
        // parse instructions
        let (input, _) = many0_count(
            | input: Tokens<'a> | Parser::parse_instruction(input, builder.clone()))(input)?;
        // fixup terminator
        let (input, _) = Parser::parse_terminator(input, builder.clone())?;
        Ok((input, handler))
    }

    fn parse_function_body(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        let (input, bb_list) = many1(
            | input: Tokens<'a> | Parser::parse_basic_block(input, builder.clone()))(input)?;
        Ok((input, ()))
    }

    fn parse_function_header(
        input: Tokens<'a>
    ) -> IResult<Tokens<'a>, (String, Vec<(Option<String>, Type)>, Type, bool)> {
        tuple((
            // keyword `fn` and function identifier
            preceded(token(Token::KwFn), map(identifier, String::from)),
            // parameter names and type
            delimited(
                token(Token::LParen),
                separated_list0(
                    token(Token::Colon),
                    pair(
                        map(identifier, | name | Some(String::from(name))),
                        preceded(token(Token::Colon), parse_type))
                ),
                token(Token::RParen)
            ),
            // return type
            preceded(token(Token::Arrow), parse_type),
            // peek the next token to decide it is a
            // function declaration or definition.
            peek(alt((
                value(true, token(Token::SemiColon)),
                value(false, token(Token::LBrace))
            )))
        ))(input)
    }

    fn parse_function(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, ()> {
        let (input, (name, params, ret, external)) = Parser::parse_function_header(input)?;
        builder
            .borrow_mut()
            .emit_function(name, params, ret, external);
        alt((
            value((), token(Token::SemiColon)),
            | input: Tokens<'a> | Parser::parse_function_body(input, builder.clone())
        ))(input)
    }

    pub fn parse_from_complete_input(
        input: Tokens<'a>,
        builder: Rc<RefCell<IRBuilder>>
    ) -> IResult<Tokens<'a>, Module> {
        let (input, _) = many0(
            | input: Tokens<'a> | Parser::parse_function(input, builder.clone())
        )(input).unwrap();
        Ok((input, builder.borrow().module.clone()))
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

    #[test]
    fn test_filter_comment() {
        test_parser!(parse_type, "fn((), // EOL-style comment \n  i64) -> /* C++ style comment \n * newline */ i64*",
                Type::get_function(vec![Type::get_unit(), Type::get_i64()], Type::get_pointer(Type::get_i64())));
    }
}


