use strum_macros::AsRefStr;
use nom::error::{ErrorKind, ParseError};
use nom::{IResult, Err};

#[derive(Debug, PartialEq, AsRefStr)]
pub enum CustomError<I> {
    XPST0003,
    XQST0090,
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for CustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        CustomError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<nom::Err<CustomError<I>>> for CustomError<I> {
    fn from(error: Err<CustomError<I>>) -> Self {
        match error {
            Err::Incomplete(e) => todo!(),
            Err::Error(e) => e,
            Err::Failure(e) => e,
        }
    }
}

pub trait IResultExt<I, O, E> {
    fn or_failure(self, error: CustomError<I>) -> IResult<I, O, E>;
}

impl<I, O> IResultExt<I, O, CustomError<I>> for IResult<I, O, CustomError<I>> {
    fn or_failure(self, error: CustomError<I>) -> IResult<I, O, CustomError<I>> {
        if self.is_ok() {
            self
        } else {
            Err(nom::Err::Failure(error))
        }
        // match self {
        //     Ok(res) => Ok(res),
        //     Err(..) => Err(nom::Err::Failure(error)),
        //     // Err(..) => Err(nom::Err::Error(nom::error::ParseError::from_char("", ' '))),
        // }
    }
}

// impl<I, O, E> Finish<I, O, E> for IResult<I, O, E> {
//     fn finish(self) -> Result<(I, O), E> {
//         match self {
//             Ok(res) => Ok(res),
//             Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(e),
//             Err(Err::Incomplete(_)) => {
//                 panic!("Cannot call `finish()` on `Err(Err::Incomplete(_))`: this result means that the parser does not have enough data to decide, you should gather more data and try to reapply  the parser instead")
//             }
//         }
//     }
// }

// fn error<'a, T>(check: IResult<&'a str, T, CustomError<&'a str>>, error: CustomError<&'a str>) -> IResult<&'a str, T, CustomError<&'a str>> {
//     if check.is_ok() {
//         check
//     } else {
//         Err(nom::Err::Failure(error))
//     }
// }

pub(crate) fn failure<'a, T>(error: CustomError<&'a str>, check: IResult<&'a str, T, CustomError<&'a str>>) -> IResult<&'a str, T, CustomError<&'a str>> {
    if check.is_ok() {
        check
    } else {
        Err(nom::Err::Failure(error))
    }
}