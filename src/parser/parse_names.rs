use crate::parser::errors::CustomError;
use crate::values::{QName, Name};

use nom::{
    bytes::complete::{tag, take_while, take_while_m_n},
    IResult
};
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};
use crate::eval::expression::Expression;
use crate::parser::helper::ws1;
use crate::parser::parse_literal::parse_uri_qualified_name;

// [7]   	QName	   ::=   	PrefixedName | UnprefixedName
// [8]   	PrefixedName	   ::=   	Prefix ':' LocalPart
// [9]   	UnprefixedName	   ::=   	LocalPart
// [10]   	Prefix	   ::=   	NCName
// [11]   	LocalPart	   ::=   	NCName
pub(crate) fn parse_qname(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    map(
        tuple((parse_ncname, opt(preceded(tag(":"), parse_ncname)))),
        |(part1, part2)| {
            if let Some(name) = part2 {
                QName { local_part: name, url: None, prefix: Some(part1) }
            } else {
                QName { local_part: part1, url: None, prefix: None }
            }
        }
    )(input)
}

pub(crate) fn parse_qname_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, qname) = parse_qname(input)?;
    Ok((input, Box::new(qname)))
}

pub(crate) fn parse_ws1_qname_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws1(input)?;
    let (input, qname) = parse_qname(input)?;
    Ok((input, Box::new(qname)))
}

// [218]    	EQName 	   ::=    	QName | URIQualifiedName
pub(crate) fn parse_eqname(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    alt((parse_uri_qualified_name, parse_qname))(input)
}

// [4]   	NCName	   ::=   	Name - (Char* ':' Char*)	/* An XML Name, minus the ":" */
pub(crate) fn parse_ncname(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, name_start) = take_while_m_n(1, 1, is_name_start_char)(input)?;
    let (input, name_end) = take_while(is_name_char)(input)?;

    let mut name = String::new();
    name.push_str(name_start);
    name.push_str(name_end);

    Ok((input, name))
}

pub(crate) fn parse_ws1_ncname(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, _) = ws1(input)?;
    parse_ncname(input)
}

pub(crate) fn parse_ncname_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, name) = parse_ncname(input)?;
    Ok((input, Name::boxed(name) ))
}

//[4]   	NameStartChar	   ::=   	":" (An XML Name, minus the ":") | [A-Z] | "_" | [a-z] TODO: | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn is_name_start_char(c: char) -> bool {
    c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}

//[4a]   	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] TODO: | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
fn is_name_char(c: char) -> bool {
    is_name_start_char(c) || c == '-' || c == '.' || (c >= '0' && c <= '9')
}