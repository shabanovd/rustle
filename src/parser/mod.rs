use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_until, take_while, take_while1, take_while_m_n},
    character::complete::{multispace0, multispace1},
    IResult
};
use nom::character::complete::one_of;
use nom::error::{Error, ParseError};

use crate::fns::{expr_to_params, Param};
use crate::namespaces::*;

use crate::parser::errors::{CustomError, failure, IResultExt};
use crate::value::QName;
use crate::parser::op::{Expr, Operator, Representation, Statement};
use crate::parser::parse_expr::parse_main_module;
use crate::parser::helper::ws;

mod helper;
pub(crate) mod op;
mod errors;
mod macros;
mod parse_names;
mod parse_expr;
mod parse_literal;
mod parse_xml;

// [1]    	Module 	   ::=    	TODO: VersionDecl? (LibraryModule | MainModule)
pub fn parse(input: &str) -> Result<Vec<Statement>, CustomError<&str>> {

    if input.len() == 0 {
        // empty is invalid
        Err(CustomError::XPST0003)
    } else {
        let (input, program) = parse_main_module(input)?;
        if input.len() > 0 {
            // something left unparsed
            Err(CustomError::XPST0003)
        } else {
            Ok(program)
        }
    }
}