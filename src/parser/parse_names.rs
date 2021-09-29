use crate::parser::op::found_qname;
use crate::namespaces::*;
use crate::parser::errors::CustomError;
use crate::values::QName;

use nom::{
    bytes::complete::{tag, take_while, take_while_m_n},
    IResult
};
use crate::eval::expression::Expression;

fn parse_name(input: &str) -> IResult<&str, String, CustomError<&str>> {
    parse_ncname(input)
}

// [7]   	QName	   ::=   	PrefixedName | UnprefixedName
pub(crate) fn parse_qname(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    // use as workaround
    parse_eqname(input)
}

pub(crate) fn parse_qname_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, qname) = parse_qname(input)?;
    Ok((input, Box::new(qname)))
}

// [218]    	EQName 	   ::=    	QName TODO: | URIQualifiedName
pub(crate) fn parse_eqname(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    // [8]   	PrefixedName	   ::=   	Prefix ':' LocalPart
    // [9]   	UnprefixedName	   ::=   	LocalPart
    // [10]   	Prefix	   ::=   	NCName
    // [11]   	LocalPart	   ::=   	NCName

    let (input, name1) = parse_ncname(input)?;

    let check = tag(":")(input);
    if check.is_ok() {
        let (input, _) = check?;

        let (input, name2) = parse_ncname(input)?;

        let url = if name1 == String::from(XML.prefix) {
            Some(String::from(XML.url))
        } else if name1 == String::from(SCHEMA.prefix) {
            Some(String::from(SCHEMA.url))
        } else if name1 == String::from(SCHEMA_INSTANCE.prefix) {
            Some(String::from(SCHEMA_INSTANCE.url))
        } else if name1 == String::from(XPATH_FUNCTIONS.prefix) {
            Some(String::from(XPATH_FUNCTIONS.url))
        } else if name1 == String::from(XPATH_MAP.prefix) {
            Some(String::from(XPATH_MAP.url))
        } else if name1 == String::from(XPATH_ARRAY.prefix) {
            Some(String::from(XPATH_ARRAY.url))
        } else if name1 == String::from(XPATH_MATH.prefix) {
            Some(String::from(XPATH_MATH.url))
        } else if name1 == String::from(XQUERY_LOCAL.prefix) {
            Some(String::from(XQUERY_LOCAL.url))
        } else if name1 == String::from(XQT_ERROR.prefix) {
            Some(String::from(XQT_ERROR.url))
        } else {
            None
        };

        found_qname(
            input,
            QName { local_part: name2, url, prefix: Some(name1) }
        )
    } else {
        found_qname(
            input,
            QName { local_part: name1, url: None, prefix: None }
        )
    }
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

//[4]   	NameStartChar	   ::=   	":" (An XML Name, minus the ":") | [A-Z] | "_" | [a-z] TODO: | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn is_name_start_char(c: char) -> bool {
    c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}

//[4a]   	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] TODO: | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
fn is_name_char(c: char) -> bool {
    is_name_start_char(c) || c == '-' || c == '.' || (c >= '0' && c <= '9')
}