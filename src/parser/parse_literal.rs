use crate::parse_one_of;

use crate::parser::errors::{CustomError, IResultExt};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    error::Error,
    IResult
};

use crate::parser::op::found_expr;
use crate::parser::parse_xml::{parse_refs, parse_refs_as_char};
use crate::parser::helper::ws;
use ordered_float::OrderedFloat;
use bigdecimal::BigDecimal;
use nom::combinator::map;
use nom::sequence::{preceded, terminated, tuple};
use crate::eval::expression::Expression;
use crate::eval::prolog::*;
use crate::parser::parse_names::parse_ncname;
use crate::values::QName;

// [129]    	Literal 	   ::=    	NumericLiteral | StringLiteral
parse_one_of!(parse_literal,
    parse_numeric_literal, parse_string_literal,
);

// [130]    	NumericLiteral 	   ::=    	IntegerLiteral TODO: | DecimalLiteral | DoubleLiteral
// [220]    	DecimalLiteral 	   ::=    	("." Digits) | (Digits "." [0-9]*)
// [221]    	DoubleLiteral 	   ::=    	(("." Digits) | (Digits ("." [0-9]*)?)) [eE] [+-]? Digits
pub(crate) fn parse_numeric_literal(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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
        match number.as_str().parse::<f64>() {
            Ok(number) => {
                found_expr(input, Box::new(Double { number: OrderedFloat(number) }))
            },
            Err(..) => {
                Err(nom::Err::Failure(CustomError::FOAR0002))
            }
        }

    } else {
        if a == "0" {
            let number = format!("{}", b);

            match number.parse::<i128>() {
                Ok(number) => {
                    found_expr(input, Box::new(Integer { number }))
                },
                Err(..) => {
                    Err(nom::Err::Failure(CustomError::FOAR0002))
                }
            }
        } else {
            let number = format!("{}.{}", b, a);

            match number.parse::<BigDecimal>() {
                Ok(number) => {
                    found_expr(input, Box::new(Decimal { number: number.normalized() }))
                },
                Err(..) => {
                    Err(nom::Err::Failure(CustomError::FOAR0002))
                }
            }
        }
    }
}

// [219]    	IntegerLiteral 	   ::=    	Digits
pub(crate) fn parse_integer_literal(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, number) = take_while1(is_digits)(input)?;

    found_expr(input, Box::new(Integer { number: number.parse::<i128>().unwrap() }))
}

// [222]    	StringLiteral 	   ::=    	('"' (PredefinedEntityRef | CharRef | EscapeQuot | [^"&])* '"') | ("'" (PredefinedEntityRef | CharRef | EscapeApos | [^'&])* "'")
pub(crate) fn parse_string_literal(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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
            data.push(Box::new(StringExpr::from(content)));

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
                    data.push(Box::new(EscapeApos{}));
                } else {
                    data.push(Box::new(EscapeQuot{}));
                }
            } else {
                return if data.len() == 0 {
                    found_expr(current_input, Box::new(StringExpr::from("")))
                } else if data.len() == 1 {
                    let expr = data.remove(0);
                    Ok((current_input, expr))
                } else {
                    found_expr(current_input, Box::new(StringComplex { exprs: data }))
                }
            }
        }
    }
}

pub(crate) fn parse_string_literal_as_string(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, open) = alt((tag("\""), tag("'")))(input)?;
    let except = if open == "'" { "'&" } else { "\"&" };

    let mut data = vec![];

    let mut current_input = input;
    loop {
        let check = parse_refs_as_char(current_input);
        match check {
            Ok((input, expr)) => {
                current_input = input;

                data.push(expr.to_string());
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
            data.push(content.to_string());

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
                    data.push("'".to_string());
                } else {
                    data.push("\"".to_string());
                }
            } else {
                return if data.len() == 0 {
                    Ok((current_input, "".to_string()))
                } else if data.len() == 1 {
                    let str = data.remove(0);
                    Ok((current_input, str))
                } else {
                    Ok((current_input, data.join("")))
                }
            }
        }
    }
}

// ws: explicit
// [223]    	URIQualifiedName 	   ::=    	BracedURILiteral NCName
pub(crate) fn parse_uri_qualified_name(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    let (input, (qname, error)) = map(
        tuple((parse_braced_uri_literal, parse_ncname)),
        |(url, local_part)| {
            let url = url.trim();
            if url.is_empty() {
                (Some(QName { prefix: None, url: None, local_part }), None)
            } else {
                if "http://www.w3.org/2000/xmlns/" == url {
                    (None, Some(CustomError::XQST0070))
                } else {
                    (Some(QName { prefix: None, url: Some(url.to_string()), local_part }), None)
                }
            }
        }
    )(input)?;

    if let Some(code) = error {
        Err(nom::Err::Failure(code))
    } else if let Some(qname) = qname {
        Ok((input, qname))
    } else {
        Err(nom::Err::Error(CustomError::XPST0003))
    }
}

// [224]    	BracedURILiteral 	   ::=    	"Q" "{" (PredefinedEntityRef | CharRef | [^&{}])* "}"
// [225]    	PredefinedEntityRef 	   ::=    	"&" ("lt" | "gt" | "amp" | "quot" | "apos") ";"
pub(crate) fn parse_braced_uri_literal(input: &str) -> IResult<&str, String, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("Q{"))),
            terminated(parse_string, tag("}"))
        ),
        |url| {
            // workaround
            let mut old_url= url.trim()
                .replace("\t", " ")
                .replace("\n", " ")
                .replace("\r", " ");
            let mut new_url = old_url.replace("  ", " ");
            while old_url != new_url {
                old_url = new_url;
                new_url = old_url.replace("  ", " ");
            }
            new_url
        }
    )(input)
}

pub(crate) fn parse_string(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let mut data = vec![];

    let mut current_input = input;
    loop {
        let check = parse_refs_as_char(current_input);
        match check {
            Ok((input, expr)) => {
                current_input = input;

                data.push(expr.to_string());
                continue;
            },
            Err(nom::Err::Failure(e)) => {
                return Err(nom::Err::Failure(e));
            },
            _ => {}
        }

        let check = is_not("&{}")(current_input);
        current_input = if check.is_ok() {
            let (input, content) = check?;
            data.push(content.to_string());

            input
        } else {
            current_input
        };

        let check: Result<(&str, &str), nom::Err<Error<&str>>> = tag("&")(current_input);
        if check.is_err() {
            return if data.len() == 0 {
                Ok((current_input, "".to_string()))
            } else if data.len() == 1 {
                let str = data.remove(0);
                Ok((current_input, str))
            } else {
                Ok((current_input, data.join("")))
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