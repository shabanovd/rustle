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

// [5]   	Name	   ::=   	NameStartChar (NameChar)*
pub(crate) fn parse_name(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, name_start) = take_while_m_n(1, 1, is_name_start_char)(input)?;
    let (input, name_end) = take_while(is_name_char)(input)?;

    let mut name = String::new();
    name.push_str(name_start);
    name.push_str(name_end);

    Ok((input, name))
}

// [4]   	NCName	   ::=   	Name - (Char* ':' Char*)	/* An XML Name, minus the ":" */
pub(crate) fn parse_ncname(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, name_start) = take_while_m_n(1, 1, is_name_start_char_without_colon)(input)?;
    let (input, name_end) = take_while(is_name_char_without_colon)(input)?;

    let mut name = String::new();
    name.push_str(name_start);
    name.push_str(name_end);

    Ok((input, name))
}

// [7]   	Nmtoken	   ::=   	(NameChar)+
pub(crate) fn parse_nmtoken(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, token) = take_while(is_name_char)(input)?;

    Ok((input, token.to_string()))
}

pub(crate) fn parse_ws1_ncname(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, _) = ws1(input)?;
    parse_ncname(input)
}

pub(crate) fn parse_ncname_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, name) = parse_ncname(input)?;
    Ok((input, Name::boxed(name) ))
}

const CHAR_X_C0: char = char::from_u32(0xC0).unwrap();
const CHAR_X_D6: char = char::from_u32(0xD6).unwrap();
const CHAR_X_D8: char = char::from_u32(0xD8).unwrap();
const CHAR_X_F6: char = char::from_u32(0xF6).unwrap();
const CHAR_X_F8: char = char::from_u32(0xF8).unwrap();
const CHAR_X2FF: char = char::from_u32(0x2FF).unwrap();
const CHAR_X370: char = char::from_u32(0x370).unwrap();
const CHAR_X37D: char = char::from_u32(0x37D).unwrap();
const CHAR_X37F: char = char::from_u32(0x37F).unwrap();
const CHAR_X1FFF: char = char::from_u32(0x1FFF).unwrap();
const CHAR_X200C: char = char::from_u32(0x200C).unwrap();
const CHAR_X200D: char = char::from_u32(0x200D).unwrap();

const CHAR_X2070: char = char::from_u32(0x2070).unwrap();
const CHAR_X218F: char = char::from_u32(0x218F).unwrap();
const CHAR_X2C00: char = char::from_u32(0x2C00).unwrap();
const CHAR_X2FEF: char = char::from_u32(0x2FEF).unwrap();
const CHAR_X3001: char = char::from_u32(0x3001).unwrap();
const CHAR_X_D7FF: char = char::from_u32(0xD7FF).unwrap();
const CHAR_X_F900: char = char::from_u32(0xF900).unwrap();
const CHAR_X_FDCF: char = char::from_u32(0xFDCF).unwrap();
const CHAR_X_FDF0: char = char::from_u32(0xFDF0).unwrap();
const CHAR_X_FFFD: char = char::from_u32(0xFFFD).unwrap();
const CHAR_X10000: char = char::from_u32(0x10000).unwrap();
const CHAR_X_EFFFF: char = char::from_u32(0xEFFFF).unwrap();

// [4]   	NameStartChar	   ::=   	":" (An XML Name, minus the ":") | [A-Z] | "_" | [a-z]
// | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D]
// | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD]
// | [#x10000-#xEFFFF]
fn is_name_start_char(c: char) -> bool {
    c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
        || (c >= CHAR_X_C0 && c <= CHAR_X_D6)
        || (c >= CHAR_X_D8 && c <= CHAR_X_F6)
        || (c >= CHAR_X_F8 && c <= CHAR_X2FF)
        || (c >= CHAR_X370 && c <= CHAR_X37D)
        || (c >= CHAR_X37F && c <= CHAR_X1FFF)
        || (c >= CHAR_X200C && c <= CHAR_X200D)
        || (c >= CHAR_X2070 && c <= CHAR_X218F)
        || (c >= CHAR_X2C00 && c <= CHAR_X2FEF)
        || (c >= CHAR_X3001 && c <= CHAR_X_D7FF)
        || (c >= CHAR_X_F900 && c <= CHAR_X_FDCF)
        || (c >= CHAR_X_FDF0 && c <= CHAR_X_FFFD)
        || (c >= CHAR_X10000 && c <= CHAR_X_EFFFF)
}

const CHAR_X_B7: char = char::from_u32(0xB7).unwrap();
const CHAR_X0300: char = char::from_u32(0x0300).unwrap();
const CHAR_X036F: char = char::from_u32(0x036F).unwrap();
const CHAR_X203F: char = char::from_u32(0x203F).unwrap();
const CHAR_X2040: char = char::from_u32(0x2040).unwrap();

// [4a]   	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
fn is_name_char(c: char) -> bool {
    // println!("is_name_char {:?} {:X}", c, c as u32);
    is_name_start_char(c) || c == ':' || c == '-' || c == '.' || (c >= '0' && c <= '9')
        || c == CHAR_X_B7
        || (c >= CHAR_X0300 && c <= CHAR_X036F)
        || (c >= CHAR_X203F && c <= CHAR_X2040)
}

fn is_name_start_char_without_colon(c: char) -> bool {
    c != ':' && (c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
        || (c >= CHAR_X_C0 && c <= CHAR_X_D6)
        || (c >= CHAR_X_D8 && c <= CHAR_X_F6)
        || (c >= CHAR_X_F8 && c <= CHAR_X2FF)
        || (c >= CHAR_X370 && c <= CHAR_X37D)
        || (c >= CHAR_X37F && c <= CHAR_X1FFF)
        || (c >= CHAR_X200C && c <= CHAR_X200D)
        || (c >= CHAR_X2070 && c <= CHAR_X218F)
        || (c >= CHAR_X2C00 && c <= CHAR_X2FEF)
        || (c >= CHAR_X3001 && c <= CHAR_X_D7FF)
        || (c >= CHAR_X_F900 && c <= CHAR_X_FDCF)
        || (c >= CHAR_X_FDF0 && c <= CHAR_X_FFFD)
        || (c >= CHAR_X10000 && c <= CHAR_X_EFFFF)
    )
}

fn is_name_char_without_colon(c: char) -> bool {
    is_name_start_char_without_colon(c) || c == '-' || c == '.' || (c >= '0' && c <= '9')
        || c == CHAR_X_B7
        || (c >= CHAR_X0300 && c <= CHAR_X036F)
        || (c >= CHAR_X203F && c <= CHAR_X2040)
}