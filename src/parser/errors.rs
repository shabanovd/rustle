use std::fmt::Debug;
use strum_macros::AsRefStr;
use nom::error::{ErrorKind, ParseError, FromExternalError};
use nom::{IResult, Err};
use crate::eval::ErrorInfo;
use crate::values::{Type, Types};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, AsRefStr)]
pub enum ErrorCode {
    TODO,
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

    FOAP0001,
    FOAR0001,
    FOAR0002,
    FOAY0001,
    FOAY0002,
    FOCA0001,
    FOCA0002,
    FOCA0003,
    FOCA0005,
    FOCA0006,
    FOCH0001,
    FOCH0002,
    FOCH0003,
    FOCH0004,
    FODC0001,
    FODC0002,
    FODC0003,
    FODC0004,
    FODC0005,
    FODC0006,
    FODC0010,
    FODF1280,
    FODF1310,
    FODT0001,
    FODT0002,
    FODT0003,
    FOER0000,
    FOFD1340,
    FOFD1350,
    FOJS0001,
    FOJS0003,
    FOJS0004,
    FOJS0005,
    FOJS0006,
    FOJS0007,
    FONS0004,
    FONS0005,
    FOQM0001,
    FOQM0002,
    FOQM0003,
    FOQM0005,
    FOQM0006,
    FORG0001,
    FORG0002,
    FORG0003,
    FORG0004,
    FORG0005,
    FORG0006,
    FORG0008,
    FORG0009,
    FORG0010,
    FORX0001,
    FORX0002,
    FORX0003,
    FORX0004,
    FOTY0012,
    FOTY0013,
    FOTY0014,
    FOTY0015,
    FOUT1170,
    FOUT1190,
    FOUT1200,
    FOXT0001,
    FOXT0002,
    FOXT0003,
    FOXT0004,
    FOXT0006
}

impl ErrorCode {
    pub(crate) fn forg0001(obj: &dyn std::any::Any, to: Types) -> ErrorInfo {
        (ErrorCode::FORG0001, format!("{:?} cannot be cast to {:?}", obj, to))
        // (ErrorCode::FORG0001, format!("The string {:?} cannot be cast to a {}", str, type_name))
        // (ErrorCode::FORG0001, format!("can't convert to {} {:?}", type_name, str))
    }

    pub(crate) fn forg0006(obj: String) -> ErrorInfo {
        (ErrorCode::FORG0006, format!("inappropriate value {:?}", obj))
    }

    pub(crate) fn xpty0004(t: &Type, to: Types) -> ErrorInfo {
        (ErrorCode::XPTY0004, format!("{:?} cannot be cast to {:?}", t, to))
    }
}

#[derive(Debug, PartialEq, AsRefStr)]
pub enum CustomError<I> {
    XQ(I, ErrorCode),
    // XQST0039,
    // XQST0040,
    // XQST0031,
    // XQST0070,
    // XQST0087,
    // XPST0003,
    // FOAR0002,
    // XQST0090,
    // XQST0118,

    Nom(I, ErrorKind),
}

impl<I> CustomError<I> {
    pub(crate) fn new(i: I, code: ErrorCode) -> CustomError<I> {
        CustomError::XQ(i, code)
    }

    pub(crate) fn failed(i: I, code: ErrorCode) -> nom::Err<CustomError<I>> {
        nom::Err::Failure(CustomError::XQ(i, code))
    }

    pub(crate) fn error(i: I, code: ErrorCode) -> nom::Err<CustomError<I>> {
        nom::Err::Error(CustomError::XQ(i, code))
    }
}

impl<I> FromExternalError<I, CustomError<I>> for CustomError<I> {
    fn from_external_error(input: I, kind: ErrorKind, e: CustomError<I>) -> Self {
        e
    }
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
            Err::Incomplete(..) => todo!(),
            Err::Error(e) => e,
            Err::Failure(e) => e,
        }
    }
}

pub trait IResultExt<I, O, E> {
    fn or_failure(self, error: ErrorCode) -> IResult<I, O, E>;

    fn or_error(self, error: ErrorCode) -> IResult<I, O, E>;
}

impl<I, O> IResultExt<I, O, CustomError<I>> for IResult<I, O, CustomError<I>> {
    fn or_failure(self, code: ErrorCode) -> IResult<I, O, CustomError<I>> {
        match self {
            Ok(_) => self,
            Err(error) => {
                match error {
                    Err::Incomplete(e) => Err(nom::Err::Incomplete(e)),
                    Err::Error(e) |
                    Err::Failure(e) => {
                        match e {
                            CustomError::XQ(i, _) |
                            CustomError::Nom(i, _) => {
                                Err(nom::Err::Failure(CustomError::XQ(i, code)))
                            }
                        }
                    }
                }
            }
        }
    }

    fn or_error(self, code: ErrorCode) -> IResult<I, O, CustomError<I>> {
        match self {
            Ok(_) => self,
            Err(error) => {
                match error {
                    Err::Incomplete(e) => Err(nom::Err::Incomplete(e)),
                    Err::Error(e) |
                    Err::Failure(e) => {
                        match e {
                            CustomError::XQ(i, _) |
                            CustomError::Nom(i, _) => {
                                Err(nom::Err::Error(CustomError::XQ(i, code)))
                            }
                        }
                    }
                }
            }
        }
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