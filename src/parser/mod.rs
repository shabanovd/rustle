use nom::combinator::opt;
use nom::Err;
use crate::parser::errors::{CustomError, ErrorCode};
use crate::parser::helper::ws;
use crate::parser::op::Statement;
use crate::parser::parse_expr::{parse_main_module, parse_version_decl};

mod helper;
pub(crate) mod op;
pub mod errors;
mod macros;
pub(crate) mod parse_names;
mod parse_expr;
mod parse_literal;
mod parse_xml;
pub mod parse_duration;

// [1]    	Module 	   ::=    	TODO: VersionDecl? (LibraryModule | MainModule)
pub fn parse(input: &str) -> Result<Vec<Statement>, ErrorCode> {
    match parse_script(input) {
        Ok(program) => Ok(program),
        Err(code) => {
            let code = match code {
                CustomError::XQST0039 => ErrorCode::XQST0039,
                CustomError::XQST0040 => ErrorCode::XQST0040,
                CustomError::XQST0031 => ErrorCode::XQST0031,
                CustomError::XQST0070 => ErrorCode::XQST0070,
                CustomError::XQST0087 => ErrorCode::XQST0087,
                CustomError::XPST0003 => ErrorCode::XPST0003,
                CustomError::FOAR0002 => ErrorCode::FOAR0002,
                CustomError::XQST0090 => ErrorCode::XQST0090,
                CustomError::XQST0118 => ErrorCode::XQST0118,
                CustomError::Nom(_, _) => ErrorCode::XPST0003,
            };
            Err(code)
        }
    }
}

pub fn parse_script(input: &str) -> Result<Vec<Statement>, CustomError<&str>> {
    let (input, version_decl) = opt(parse_version_decl)(input)?;

    if input.len() == 0 {
        // empty is invalid
        Err(CustomError::XPST0003)
    } else {
        let (input, program) = parse_main_module(input)?;
        let (input, _) = ws(input)?;
        if input.len() > 0 {
            println!("unparsed {:?}", input);
            // something left un-parsed
            Err(CustomError::XPST0003)
        } else {
            Ok(program)
        }
    }
}
