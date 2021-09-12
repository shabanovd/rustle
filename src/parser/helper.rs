use nom::{branch::alt, bytes::complete::{is_not, tag, take_till, take_until, take_while1, take_while_m_n}, character::complete::{multispace0, multispace1}, IResult, InputTakeAtPosition, AsChar};
use crate::parser::errors::CustomError;
use nom::error::{ParseError, Error, ErrorKind};
use nom::bytes::complete::take;

pub(crate) fn ws(input: &str) -> Result<(&str, &str), nom::Err<CustomError<&str>>> {
    match find_ws_end(input) {
        Some((pos, err)) => {
            if err {
                Err(nom::Err::Failure(CustomError::XPST0003))
            } else {
                take(pos as usize)(input)
            }
        },
        None => take(0 as usize)(input)
    }
}

enum State {
    None,
    OpeningComment,
    ClosingComment
}

fn find_ws_end(input: &str) -> Option<(usize, bool)> {
    let mut open_comments = 0;
    let mut step = State::None;
    for (i, c) in input.chars().enumerate() {
        if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
            step = State::None;
        } else {
            match step {
                State::None => {
                    match c {
                        '(' => step = State::OpeningComment,
                        ':' => step = State::ClosingComment,
                        _ => if open_comments == 0 { return Some((i, false)); }
                    }
                },
                State::OpeningComment => {
                    if c == ':' { open_comments += 1 }
                    step = State::None;
                    if open_comments == 0 { return Some((i - 1, false)); }
                },
                State::ClosingComment => {
                    if c == ')' { open_comments -= 1 }
                    step = State::None;
                    if open_comments == 0 { return Some((i - 1, false)); }
                }
            }
        }
    }
    if open_comments == 0 {
        None
    } else {
        Some((0, true))
    }
}

pub(crate) fn ws_tag<'a>(token: &str, input: &'a str) -> IResult<&'a str, &'a str, CustomError<&'a str>> {
    let (input, _) = ws(input)?;
    tag(token)(input)
}

pub(crate) fn ws_tag_ws<'a>(token: &str, input: &'a str) -> IResult<&'a str, &'a str, CustomError<&'a str>> {
    let (input, _) = ws(input)?;
    let (input, _) = tag(token)(input)?;
    ws(input)
}