use nom::IResult;
use crate::parser::errors::CustomError;
use crate::values::QName;
use crate::eval::expression::Expression;

pub(crate) fn found_exprs(input: &str, exprs: Vec<Box<dyn Expression>>) -> IResult<&str, Vec<Box<dyn Expression>>, CustomError<&str>> {
    // let mut items = Vec::with_capacity(exprs.len());
    // for expr in exprs {
    //     items.push(Box::new(expr))
    // }
    // Ok((input, items))
    Ok((input, exprs))
}

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

#[derive(Clone)]
pub enum OneOrMore {
    One,
    More,
}

impl OneOrMore {
    pub(crate) fn from(str: &str) -> Self {
        match str {
            "/" => OneOrMore::One,
            "//" => OneOrMore::More,
            _ => panic!("error")
        }
    }
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ItemType {
    SequenceEmpty,
    Item,
    AtomicOrUnionType(QName)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OccurrenceIndicator {
    ExactlyOne,
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}