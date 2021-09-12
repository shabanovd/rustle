use crate::parser::errors::CustomError;
use crate::parser::helper::ws;
use crate::parser::op::Statement;
use crate::parser::parse_expr::parse_main_module;

mod helper;
pub(crate) mod op;
pub mod errors;
mod macros;
mod parse_names;
mod parse_expr;
mod parse_literal;
mod parse_xml;
pub mod parse_duration;

// [1]    	Module 	   ::=    	TODO: VersionDecl? (LibraryModule | MainModule)
pub fn parse(input: &str) -> Result<Vec<Statement>, CustomError<&str>> {

    if input.len() == 0 {
        // empty is invalid
        Err(CustomError::XPST0003)
    } else {
        let (input, program) = parse_main_module(input)?;
        let (input, _) = ws(input)?;
        if input.len() > 0 {
            println!("{:?}", input);
            // something left unparsed
            Err(CustomError::XPST0003)
        } else {
            Ok(program)
        }
    }
}
