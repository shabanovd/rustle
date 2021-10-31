use nom::IResult;
use crate::parser::errors::CustomError;
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