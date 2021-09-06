use crate::parser::errors::{CustomError};
use crate::parser::op::{Statement};
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