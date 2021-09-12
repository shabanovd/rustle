use strum_macros::AsRefStr;
use nom::error::{ErrorKind, ParseError};
use nom::{IResult, Err};

#[derive(Debug, PartialEq, AsRefStr)]
pub enum ErrorCode {
    XPST0001,
    XPDY0002,
    XPST0003,
    XPTY0004,
    XPST0005,
    XPST0008,
    XQST0009,
    XQST0012,
    XQST0013,
    XQST0016,
    XPST0017,
    XPTY0018,
    XPTY0019,
    XPTY0020,
    XQST0022,
    XQTY0024,
    XQDY0025,
    XQDY0026,
    XQDY0027,
    XQTY0030,
    XQST0031,
    XQST0032,
    XQST0033,
    XQST0034,
    XQST0035,
    XQST0038,
    XQST0039,
    XQST0040,
    XQDY0041,
    XQDY0044,
    XQST0045,
    XQST0046,
    XQST0047,
    XQST0048,
    XQST0049,
    XPDY0050,
    XPST0051,
    XQST0052,
    XQDY0054,
    XQST0055,
    XQST0057,
    XQST0058,
    XQST0059,
    XQST0060,
    XQDY0061,
    XQDY0064,
    XQST0065,
    XQST0066,
    XQST0067,
    XQST0068,
    XQST0069,
    XQST0070,
    XQST0071,
    XQDY0072,
    XQDY0074,
    XQST0075,
    XQST0076,
    XQST0079,
    XPST0080,
    XPST0081,
    XQDY0084,
    XQST0085,
    XQTY0086,
    XQST0087,
    XQST0088,
    XQST0089,
    XQST0090,
    XQDY0091,
    XQDY0092,
    XQST0094,
    XQDY0096,
    XQST0097,
    XQST0098,
    XQST0099,
    XQDY0101,
    XQDY0102,
    XQST0103,
    XQST0104,
    XQTY0105,
    XQST0106,
    XQST0108,
    XQST0109,
    XQST0110,
    XQST0111,
    XQST0113,
    XQST0114,
    XQST0115,
    XQST0116,
    XPTY0117,
    XQST0118,
    XQST0119,
    XQST0125,
    XQST0129,
    XPDY0130,
    XQST0134,
    XQDY0137,
}

#[derive(Debug, PartialEq, AsRefStr)]
pub enum CustomError<I> {
    XPST0003,
    FOAR0002,
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
            match self {
                Err(nom::Err::Error(CustomError::Nom(i,t))) |
                Err(nom::Err::Failure(CustomError::Nom(i,t))) => {
                    println!("ERROR: {:?}", t);
                }
                _ => {}
            }
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