use nom::IResult;
use crate::eval::comparison::ValueOrdering;
use crate::eval::ErrorInfo;
use crate::parser::errors::{CustomError, ErrorCode};
use crate::values::QName;
use crate::eval::expression::Expression;

pub(crate) fn found_expr(input: &str, expr: Box<dyn Expression>) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    // if DEBUG {
    //     println!("\nfound: {:?}\ninput: {:?}", expr, input);
    // }
    // Ok((input, Box::new(expr)))
    Ok((input, expr))
}

pub(crate) fn found_qname(input: &str, qname: QName) -> IResult<&str, QName, CustomError<&str>> {
    // if DEBUG {
    //     println!("\nfound: {:?}\ninput: {:?}", qname, input);
    // }
    Ok((input, qname))
}

pub enum Statement {
    Prolog(Vec<Box<dyn Expression>>),
    Program(Box<dyn Expression>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Representation {
    Hexadecimal,
    Decimal
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OperatorArithmetic {
    Plus,
    Minus,
    Multiply,
    Divide,
    IDivide,
    Mod,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OperatorComparison {
    GeneralEquals,
    GeneralNotEquals,
    GeneralLessThan,
    GeneralLessOrEquals,
    GeneralGreaterThan,
    GeneralGreaterOrEquals,

    ValueEquals,
    ValueNotEquals,
    ValueLessThan,
    ValueLessOrEquals,
    ValueGreaterThan,
    ValueGreaterOrEquals,

    NodeIs,
    NodePrecedes,
    NodeFollows
}

impl OperatorComparison {
    pub(crate) fn to_comparison(&self) -> Comparison {
        match self {
            OperatorComparison::GeneralEquals => Comparison::Equals,
            OperatorComparison::GeneralNotEquals => Comparison::NotEquals,
            OperatorComparison::GeneralLessThan => Comparison::LessThan,
            OperatorComparison::GeneralLessOrEquals => Comparison::LessOrEquals,
            OperatorComparison::GeneralGreaterThan => Comparison::GreaterThan,
            OperatorComparison::GeneralGreaterOrEquals => Comparison::GreaterOrEquals,
            OperatorComparison::ValueEquals => Comparison::Equals,
            OperatorComparison::ValueNotEquals => Comparison::NotEquals,
            OperatorComparison::ValueLessThan => Comparison::LessThan,
            OperatorComparison::ValueLessOrEquals => Comparison::LessOrEquals,
            OperatorComparison::ValueGreaterThan => Comparison::GreaterThan,
            OperatorComparison::ValueGreaterOrEquals => Comparison::GreaterOrEquals,
            OperatorComparison::NodeIs |
            OperatorComparison::NodePrecedes |
            OperatorComparison::NodeFollows => panic!("internal error")
        }
    }
}

pub enum Comparison {
    Equals,
    NotEquals,
    LessThan,
    LessOrEquals,
    GreaterThan,
    GreaterOrEquals,
}

impl Comparison {
    pub(crate) fn is_it(&self, cmp_result: ValueOrdering) -> Result<bool, ErrorInfo> {
        match self {
            Comparison::Equals => {
                match cmp_result {
                    ValueOrdering::Equal |
                    ValueOrdering::QNameEqual => Ok(true),
                    _ => Ok(false),
                }
            }
            Comparison::NotEquals => {
                match cmp_result {
                    ValueOrdering::Less |
                    ValueOrdering::Greater |
                    ValueOrdering::AlwaysNotEqual |
                    ValueOrdering::QNameNotEqual => Ok(true),
                    _ => Ok(false),
                }
            }
            Comparison::LessThan => {
                match cmp_result {
                    ValueOrdering::QNameEqual |
                    ValueOrdering::QNameNotEqual => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    // None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    ValueOrdering::Less => Ok(true),
                    _ => Ok(false),
                }
            }
            Comparison::LessOrEquals => {
                match cmp_result {
                    ValueOrdering::QNameEqual |
                    ValueOrdering::QNameNotEqual => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    // None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    ValueOrdering::Equal |
                    ValueOrdering::Less => Ok(true),
                    _ => Ok(false),
                }
            }
            Comparison::GreaterThan => {
                match cmp_result {
                    ValueOrdering::QNameEqual |
                    ValueOrdering::QNameNotEqual => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    // None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    ValueOrdering::Greater => Ok(true),
                    _ => Ok(false),
                }
            }
            Comparison::GreaterOrEquals => {
                match cmp_result {
                    ValueOrdering::QNameEqual |
                    ValueOrdering::QNameNotEqual => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    // None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
                    ValueOrdering::Equal |
                    ValueOrdering::Greater => Ok(true),
                    _ => Ok(false),
                }
            }
        }
    }
}