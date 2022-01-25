use crate::parse_one_of;

use crate::parser::op::{Representation, found_expr};
use crate::parser::errors::{CustomError, ErrorCode, IResultExt};
use crate::parser::parse_literal::{is_digits, is_0_9a_f};

use nom::{branch::alt, bytes::complete::{is_not, tag, take_until, take_while1}, character::complete::multispace1, error::Error, IResult, InputTakeAtPosition, FindToken};

use crate::parser::helper::{ws, s_tag_s};
use crate::parser::parse_names::{parse_qname_expr, parse_qname, parse_ws1_qname_expr, parse_ws1_ncname};
use crate::parser::parse_expr::{parse_enclosed_expr, parse_expr};
use nom::error::ParseError;
use crate::eval::prolog::*;
use crate::eval::expression::Expression;
use nom::character::complete::multispace0;
use nom::sequence::preceded;
use crate::parser::errors::ErrorCode::*;

// [140]    	NodeConstructor 	   ::=    	DirectConstructor | ComputedConstructor
parse_one_of!(parse_node_constructor,
    parse_direct_constructor, parse_computed_constructor,
);

// TODO:
// [141]    	DirectConstructor 	   ::=    	DirElemConstructor | DirCommentConstructor | DirPIConstructor
// [142]    	DirElemConstructor 	   ::=    	"<" QName DirAttributeList ("/>" | (">" DirElemContent* "</" QName S? ">")) // ws: explicit
// [149]    	DirCommentConstructor 	   ::=    	"<!--" DirCommentContents "-->" // ws: explicit
// [150]    	DirCommentContents 	   ::=    	((Char - '-') | ('-' (Char - '-')))* // ws: explicit
// [151]    	DirPIConstructor 	   ::=    	"<?" PITarget (S DirPIContents)? "?>" // ws: explicit
// [152]    	DirPIContents 	   ::=    	(Char* - (Char* '?>' Char*)) // ws: explicit
pub(crate) fn parse_direct_constructor(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let input = tag("<")(input)?.0;

    // DirCommentConstructor
    let result = tag("!--")(input);
    if result.is_ok() {
        let (input, _) = result?;
        let (input, content) = take_until("-->")(input).or_failure(ErrorCode::XPST0003)?;

        let input = tag("-->")(input).or_failure(ErrorCode::XPST0003)?.0;

        //TODO: raise error if content end by '-'

        return found_expr(
            input,
            Box::new(NodeComment::from(content))
        );
    }

    // DirPIConstructor
    let result = tag("?")(input);
    if result.is_ok() {
        let input = result?.0;

        let (input, target) = parse_qname_expr(input).or_failure(ErrorCode::XPST0003)?;
        let (input, _) = ws(input)?;

        //TODO: target must not be 'xml'

        let (input, content) = take_until("?>")(input).or_failure(ErrorCode::XPST0003)?;

        let input = tag("?>")(input).or_failure(ErrorCode::XPST0003)?.0;

        let content = Box::new(StringExpr::from(content));

        return found_expr(input, Box::new(NodePI { target, content }))
    }

    // DirElemConstructor

    // "<" QName DirAttributeList ("/>" | (">" DirElemContent* "</" QName S? ">"))

    let (input, tag_name) = parse_qname(input).or_failure(ErrorCode::XPST0003)?;

    let (input, attributes) = parse_attribute_list(input)?;

    let mut children = Vec::new();

    let mut current_input = input;

    let check = tag("/>")(current_input);
    if check.is_ok() {
        current_input = check?.0;

    } else {
        current_input = tag(">")(current_input).or_failure(ErrorCode::XPST0003)?.0;
        loop {
            let check_for_close = tag("</")(current_input);
            if check_for_close.is_ok() {
                let (_,_) = check_for_close?;
                break;
            }

            let check = parse_dir_elem_content(current_input);
            match check {
                Ok(_) => {
                    let (input, child) = check?;
                    current_input = input;

                    children.push(child);
                },
                Err(nom::Err::Failure(_)) => return check,
                _ => break
            }
        }
        current_input = tag("</")(current_input).or_failure(ErrorCode::XPST0003)?.0;

        let (input, close_tag_name) = parse_qname(current_input).or_failure(ErrorCode::XPST0003)?;

        if close_tag_name != tag_name {
            return Err(CustomError::failed(input, XQST0118));
        }

        current_input = multispace0(input)?.0;

        current_input = tag(">")(current_input).or_failure(ErrorCode::XPST0003)?.0;
    };

    found_expr(current_input, Box::new(NodeElement { name: Box::new(tag_name), attributes, children }))
}

// [143]    	DirAttributeList 	   ::=    	(S (QName S? "=" S? DirAttributeValue)?)* // ws: explicit
pub(crate) fn parse_attribute_list(input: &str) -> IResult<&str, Option<Attributes>, CustomError<&str>> {
    let mut attributes = Attributes::new();

    let mut current_input = input;

    loop {
        let check = multispace1(current_input);
        if check.is_err() {
            break;
        }
        current_input = check?.0;

        let check = parse_qname(current_input);
        if check.is_ok() {
            let (input, name) = check?;

            let input = s_tag_s("=", input).or_failure(ErrorCode::XPST0003)?.0;

            let (input, value) = parse_dir_attribute_value(input).or_failure(ErrorCode::XPST0003)?;
            current_input = input;

            match attributes.add(name, value) {
                Err((code, msg)) => return Err(CustomError::failed(input, XQST0040)),
                _ => {}
            }
        } else {
            break;
        }
    }

    if attributes.len() == 0 {
        Ok((current_input, None))
    } else {
        Ok((current_input, Some(attributes)))
    }
}

// [144]    	DirAttributeValue 	   ::=    	('"' (EscapeQuot | QuotAttrValueContent)* '"') | ("'" (EscapeApos | AposAttrValueContent)* "'") // ws: explicit
// [145]    	QuotAttrValueContent 	   ::=    	QuotAttrContentChar | CommonContent
// [146]    	AposAttrValueContent 	   ::=    	AposAttrContentChar | CommonContent
// [229]    	QuotAttrContentChar 	   ::=    	(Char - ["{}<&])
// [230]    	AposAttrContentChar 	   ::=    	(Char - ['{}<&])
pub(crate) fn parse_dir_attribute_value(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, open) = alt((tag("\""), tag("'")))(input)?;
    let except = if open == "'" { "'{}<&" } else { "\"{}<&" };

    let mut data: Vec<Box<dyn Expression>> = vec![];
    let mut begin_input = input;
    let mut current_input = input;

    loop {
        begin_input = current_input;

        // let (input, string) = is_not(except)(current_input)
        //     .or_failure(CustomError::XPST0003)?;
        let (input, string) = current_input.split_at_position_complete(|c| except.find_token(c))?;
        current_input = input;

        if string.len() > 0 {
            data.push(Box::new(StringExpr::from(string)));
        }

        let check = parse_common_content(current_input);
        if check.is_ok() {
            let (input, expr) = check?;
            current_input = input;

            data.push(expr);
        } else {
            let (input, _) = tag(open)(current_input).or_failure(ErrorCode::XPST0003)?;
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
                    found_expr(current_input, StringExpr::empty())
                } else if data.len() == 1 {
                    let expr = data.remove(0);
                    Ok((current_input, expr))
                } else {
                    found_expr(current_input, StringComplex::boxed(data))
                }
            }
        }

        if current_input == begin_input {
            return Err(CustomError::failed(input, XPST0003));
        }
    }
}

// [147]    	DirElemContent 	   ::=    	DirectConstructor | CDataSection | CommonContent | ElementContentChar
pub(crate) fn parse_dir_elem_content(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    alt((
        parse_direct_constructor, parse_cdata_section, parse_common_content, parse_element_content_char
    ))(input)
}

// [148]    	CommonContent 	   ::=    	PredefinedEntityRef | CharRef | "{{" | "}}" | EnclosedExpr
fn parse_common_content(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    alt((parse_predefined_entity_ref, parse_char_ref, parse_curly_brackets, parse_enclosed_expr))(input)
}

pub(crate) fn parse_curly_brackets(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, char) = alt((tag("{{"), tag("}}")))(input)?;

    Ok((input, Box::new(StringExpr::from(char))))
}

// [153]    	CDataSection 	   ::=    	"<![CDATA[" CDataSectionContents "]]>" // ws: explicit
// [154]    	CDataSectionContents 	   ::=    	(Char* - (Char* ']]>' Char*)) // ws: explicit
pub(crate) fn parse_cdata_section(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("<![CDATA[")(input)?;

    let (input, content) = take_until("]]>")(input)?;

    let (input, _) = tag("]]>")(input)?;

    found_expr(input, Box::new(NodeComment::from(content)))
}

// [155]    	ComputedConstructor 	   ::=    	CompDocConstructor
// | CompElemConstructor
// | CompAttrConstructor
// | CompNamespaceConstructor
// | CompTextConstructor
// | CompCommentConstructor
// | CompPIConstructor
// [156]    	CompDocConstructor 	   ::=    	"document" EnclosedExpr
// [157]    	CompElemConstructor 	   ::=    	"element" (EQName | ("{" Expr "}")) EnclosedContentExpr
// [158]    	EnclosedContentExpr 	   ::=    	EnclosedExpr
// [159]    	CompAttrConstructor 	   ::=    	"attribute" (EQName | ("{" Expr "}")) EnclosedExpr
// [160]    	CompNamespaceConstructor 	   ::=    	"namespace" (Prefix | EnclosedPrefixExpr) EnclosedURIExpr
// [161]    	Prefix 	   ::=    	NCName
// [162]    	EnclosedPrefixExpr 	   ::=    	EnclosedExpr
// [163]    	EnclosedURIExpr 	   ::=    	EnclosedExpr
// [164]    	CompTextConstructor 	   ::=    	"text" EnclosedExpr
// [165]    	CompCommentConstructor 	   ::=    	"comment" EnclosedExpr
// [166]    	CompPIConstructor 	   ::=    	"processing-instruction" (NCName | ("{" Expr "}")) EnclosedExpr
pub(crate) fn parse_computed_constructor(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, name) = preceded(
        ws,
        alt((
            tag("document"),
            tag("element"),
            tag("attribute"),
            tag("namespace"),
            tag("text"),
            tag("comment"),
            tag("processing-instruction")
        ))
    )(input)?;

    let (input, expr) = match name {
        "document" => {
            let (input, expr) = parse_enclosed_expr(input)?;
            (input, NodeDocument::new(expr))
        }
        "text" => {
            let (input, expr) = parse_enclosed_expr(input)?;
            (input, NodeText::new(expr))
        }
        "comment" => {
            let (input, expr) = parse_enclosed_expr(input)?;
            (input, NodeComment::new(expr))
        }
        "element" => {
            let check = parse_ws1_qname_expr(input);
            let (input, name) = if check.is_ok() {
                check?
            } else {
                let (input, _) = tag("{")(input).or_failure(ErrorCode::XPST0003)?;

                let (input, expr) = parse_expr(input)?;

                let (input, _) = tag("}")(input).or_failure(ErrorCode::XPST0003)?;

                (input, expr)
            };

            let (input, expr) = parse_enclosed_expr(input)?;

            let mut children = Vec::with_capacity(1);
            children.push(expr);

            (input, NodeElement::new(name, None, children))
        }
        "attribute" => {
            let check = parse_ws1_qname_expr(input);
            let (input, name) = if check.is_ok() {
                check?
            } else {
                let (input, _) = tag("{")(input)?; // .or_error(CustomError::XPST0003)?;

                let (input, expr) = parse_expr(input)?;

                let (input, _) = tag("}")(input).or_failure(ErrorCode::XPST0003)?;

                (input, expr)
            };

            let (input, value) = parse_enclosed_expr(input)?;

            (input, NodeAttribute::new(name, value))
        }
        "namespace" => {
            // "namespace" (Prefix | EnclosedPrefixExpr) EnclosedURIExpr
            let check = parse_ws1_ncname(input);
            let (input, prefix) = if check.is_ok() {
                let (input, name) = check?;

                (input, StringExpr::new(name))
            } else {
                parse_enclosed_expr(input)?
            };

            let (input, url) = parse_enclosed_expr(input)?;

            (input, NodeNS::boxed(prefix, url))
        }
        "processing-instruction" => {
            // "processing-instruction" (NCName | ("{" Expr "}")) EnclosedExpr
            let check = parse_ws1_ncname(input);
            let (input, target) = if check.is_ok() {
                let (input, name) = check?;

                (input, StringExpr::new(name))
            } else {
                let (input, _) = tag("{")(input).or_failure(ErrorCode::XPST0003)?;

                let (input, expr) = parse_expr(input)?;

                let (input, _) = tag("}")(input).or_failure(ErrorCode::XPST0003)?;

                (input, expr)
            };

            let (input, content) = parse_enclosed_expr(input)?;

            (input, NodePI::new(target, content))
        }
        _ => panic!("internal error")
    };

    found_expr(input, expr)
}

// raise error if "nothing" after '&'
pub(crate) fn parse_refs(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let must_have: Result<(&str, &str), nom::Err<Error<&str>>> = tag("&")(input);

    let check = alt((parse_predefined_entity_ref, parse_char_ref))(input);
    match check {
        Ok(r) => {
            return Ok(r);
        },
        Err(nom::Err::Failure(e)) => return Err(nom::Err::Failure(e)),
        _ => {}
    }

    if must_have.is_ok() {
        Err(CustomError::failed(input, XPST0003))
    } else {
        Err(nom::Err::Error(ParseError::from_char(input, '&')))
    }
}

pub(crate) fn parse_refs_as_char(input: &str) -> IResult<&str, char, CustomError<&str>> {
    let must_have: Result<(&str, &str), nom::Err<Error<&str>>> = tag("&")(input);

    let check = alt((parse_predefined_entity_ref_as_char, parse_char_ref_as_char))(input);
    match check {
        Ok(r) => return Ok(r),
        Err(nom::Err::Failure(e)) => return Err(nom::Err::Failure(e)),
        _ => {}
    }

    if must_have.is_ok() {
        Err(CustomError::failed(input, XPST0003))
    } else {
        Err(nom::Err::Error(ParseError::from_char(input, '&')))
    }
}

// [66]   	CharRef	   ::=   	'&#' [0-9]+ ';' | '&#x' [0-9a-fA-F]+ ';'

// https://www.w3.org/TR/xml/#NT-Char
// [2]   	Char	   ::=   	#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
// /* any Unicode character, excluding the surrogate blocks, FFFE, and FFFF. */
pub(crate) fn parse_char_ref(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("&#")(input)?;

    let check = tag("x")(input);
    let (input, reference, representation) = if check.is_ok() {
        let (input, _) = check?;

        let (input, code) = take_while1(is_0_9a_f)(input).or_failure(ErrorCode::XPST0003)?;

        let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

        (input, u32::from_str_radix(code, 16), Representation::Hexadecimal)

    } else {
        let (input, code) = take_while1(is_digits)(input).or_failure(ErrorCode::XPST0003)?;

        let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

        (input, u32::from_str_radix(code, 10), Representation::Decimal)
    };

    if reference.is_ok() {
        let reference = reference.unwrap();
        if reference == 0x9
            || reference == 0xA
            || reference == 0xD
            || (reference >= 0x20 && reference <= 0xD7FF)
            || (reference >= 0xE000 && reference <= 0xFFFD)
            || (reference >= 0x10000 && reference <= 0x10FFFF)
        {
            found_expr(input, Box::new(CharRef { representation, reference }))
        } else {
            Err(CustomError::failed(input, XQST0090))
        }
    } else {
        Err(CustomError::failed(input, XQST0090))
    }
}

pub(crate) fn parse_char_ref_as_char(input: &str) -> IResult<&str, char, CustomError<&str>> {
    let (input, _) = tag("&#")(input)?;

    let check = tag("x")(input);
    let (input, reference, representation) = if check.is_ok() {
        let (input, _) = check?;

        let (input, code) = take_while1(is_0_9a_f)(input).or_failure(ErrorCode::XPST0003)?;

        let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

        (input, u32::from_str_radix(code, 16), Representation::Hexadecimal)

    } else {
        let (input, code) = take_while1(is_digits)(input).or_failure(ErrorCode::XPST0003)?;

        let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

        (input, u32::from_str_radix(code, 10), Representation::Decimal)
    };

    if reference.is_ok() {
        let reference = reference.unwrap();
        if reference == 0x9
            || reference == 0xA
            || reference == 0xD
            || (reference >= 0x20 && reference <= 0xD7FF)
            || (reference >= 0xE000 && reference <= 0xFFFD)
            || (reference >= 0x10000 && reference <= 0x10FFFF)
        {
            if let Some(ch) = char::from_u32(reference) {
                return Ok((input, ch))
            }
        }
    }
    Err(CustomError::failed(input, XQST0090))
}

// [225]    	PredefinedEntityRef 	   ::=    	"&" ("lt" | "gt" | "amp" | "quot" | "apos") ";"
pub(crate) fn parse_predefined_entity_ref(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("&")(input)?;

    let (input, name) = alt((
        tag("lt"),
        tag("gt"),
        tag("amp"),
        tag("quot"),
        tag("apos")
    ))(input)?;

    let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

    found_expr(input, EntityRef::boxed(name))
}

pub(crate) fn parse_predefined_entity_ref_as_char(input: &str) -> IResult<&str, char, CustomError<&str>> {
    let (input, _) = tag("&")(input)?;

    let (input, name) = alt((
        tag("lt"),
        tag("gt"),
        tag("amp"),
        tag("quot"),
        tag("apos")
    ))(input)?;

    let (input, _) = tag(";")(input).or_failure(ErrorCode::XPST0003)?;

    let ch = match name {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "quot" => '"',
        "apos" => '\'',
        _ => panic!("internal error")
    };

    Ok((input, ch))
}

pub(crate) fn parse_predefined_entity_ref_as_string(input: &str) -> IResult<&str, String, CustomError<&str>> {
    let (input, ch) = parse_predefined_entity_ref_as_char(input)?;
    Ok((input, ch.to_string()))
}

// [228]    	ElementContentChar 	   ::=    	(Char - [{}<&])
pub(crate) fn parse_element_content_char(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, content) = is_not("{}<&")(input)?;

    found_expr(input, NodeText::new(Box::new(StringExpr::from(content))))
}