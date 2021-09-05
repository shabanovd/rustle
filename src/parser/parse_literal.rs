use crate::parse_one_of;

use crate::parser::errors::{CustomError, IResultExt};
use crate::parser::op::Expr;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_until, take_while, take_while1, take_while_m_n},
    character::complete::{multispace0, multispace1, one_of},
    error::Error,
    IResult
};

use crate::parser::op::found_expr;
use crate::parser::parse_xml::parse_refs;
use crate::parser::helper::ws;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;

// [129]    	Literal 	   ::=    	NumericLiteral | StringLiteral
parse_one_of!(parse_literal, Expr,
    parse_numeric_literal, parse_string_literal,
);

// [130]    	NumericLiteral 	   ::=    	IntegerLiteral TODO: | DecimalLiteral | DoubleLiteral
// [220]    	DecimalLiteral 	   ::=    	("." Digits) | (Digits "." [0-9]*)
// [221]    	DoubleLiteral 	   ::=    	(("." Digits) | (Digits ("." [0-9]*)?)) [eE] [+-]? Digits
pub(crate) fn parse_numeric_literal(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let check = tag(".")(input);
    let (input, b, a) = if check.is_ok() {
        let (input, _) = check?;

        let (input, a) = take_while1(is_digits)(input)?;

        (input, "0", a)
    } else {
        let (input, b) = take_while1(is_digits)(input)?;

        let check: Result<(&str, &str), nom::Err<Error<&str>>> = tag(".")(input);
        if check.is_ok() {
            let (input, _) = check.unwrap();
            let check = take_while1(is_digits)(input);
            if check.is_ok() {
                let (input, a) = check?;
                (input, b, a)
            } else {
                (input, b, "0")
            }

        } else {
            (input, b, "0")
        }
    };

    let check = alt((tag("e"), tag("E")))(input);
    if check.is_ok() {
        let (input, _) = check?;

        let check = alt((tag("+"), tag("-")))(input);
        let (input, sign) = if check.is_ok() {
            let (input, sign) = check?;
            (input, sign)
        } else {
            (input, "+")
        };

        let (input, e) = take_while1(is_digits)(input)
            .or_failure(CustomError::XPST0003)?;

        let number = format!("{}.{}e{}{}", b, a, sign, e);

        // double
        match Decimal::from_scientific(number.as_str()) {
            Ok(number) => {
                found_expr(input, Expr::Double(number.normalize()))
            },
            Err(e) => {
                Err(nom::Err::Failure(CustomError::FOAR0002))
            }
        }

    } else {
        if a == "0" {
            let number = format!("{}", b);

            match number.parse::<i128>() {
                Ok(number) => {
                    found_expr(input, Expr::Integer(number))
                },
                Err(e) => {
                    Err(nom::Err::Failure(CustomError::FOAR0002))
                }
            }
        } else {
            let number = format!("{}.{}", b, a);

            match number.parse::<Decimal>() {
                Ok(number) => {
                    found_expr(input, Expr::Decimal(number.normalize()))
                },
                Err(e) => {
                    Err(nom::Err::Failure(CustomError::FOAR0002))
                }
            }
        }
    }
}

// [219]    	IntegerLiteral 	   ::=    	Digits
pub(crate) fn parse_integer_literal(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, number) = take_while1(is_digits)(input)?;

    Ok((
        input,
        Expr::Integer(number.parse::<i128>().unwrap())
    ))
}

// [222]    	StringLiteral 	   ::=    	('"' (PredefinedEntityRef | CharRef | EscapeQuot | [^"&])* '"') | ("'" (PredefinedEntityRef | CharRef | EscapeApos | [^'&])* "'")
pub(crate) fn parse_string_literal(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, open) = alt((tag("\""), tag("'")))(input)?;
    let except = if open == "'" { "'&" } else { "\"&" };

    let mut data = vec![];

    let mut current_input = input;
    loop {
        let check = parse_refs(current_input);
        match check {
            Ok((input, expr)) => {
                current_input = input;

                data.push(expr);
                continue;
            },
            Err(nom::Err::Failure(e)) => {
                return Err(nom::Err::Failure(e));
            },
            _ => {}
        }

        let check = is_not(except)(current_input);
        current_input = if check.is_ok() {
            let (input, content) = check?;
            data.push(Expr::String(String::from(content)));

            input
        } else {
            current_input
        };

        let check: Result<(&str, &str), nom::Err<Error<&str>>> = tag("&")(current_input);
        if check.is_err() {

            let (input, _) = tag(open)(current_input).or_failure(CustomError::XPST0003)?;
            current_input = input;

            // lookahead
            let check = tag(open)(current_input);
            if check.is_ok() {
                let (input, _) = check?;
                current_input = input;

                if open == "'" {
                    data.push(Expr::EscapeApos);
                } else {
                    data.push(Expr::EscapeQuot);
                }
            } else {
                return if data.len() == 0 {
                    Ok((current_input, Expr::String(String::new())))
                } else if data.len() == 1 {
                    let expr = data.remove(0);
                    Ok((current_input, expr))
                } else {
                    Ok((current_input, Expr::StringComplex(data)))
                }
            }
        }
    }
}

//[238]    	Digits 	   ::=    	[0-9]+
pub(crate) fn is_digits(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub(crate) fn is_0_9a_f(c: char) -> bool {
    (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')
}