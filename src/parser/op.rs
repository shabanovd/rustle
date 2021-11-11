use nom::IResult;
use crate::eval::comparison::ValueOrdering;
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
    pub(crate) fn is_it(&self, cmp_result: ValueOrdering) -> bool {
        match self {
            Comparison::Equals => {
                match cmp_result {
                    ValueOrdering::Equal |
                    ValueOrdering::QNameEqual => true,
                    _ => false,
                }
            }
            Comparison::NotEquals => {
                match cmp_result {
                    ValueOrdering::Less |
                    ValueOrdering::Greater |
                    ValueOrdering::AlwaysNotEqual |
                    ValueOrdering::QNameNotEqual => true,
                    _ => false,
                }
            }
            Comparison::LessThan => todo!(),
            Comparison::LessOrEquals => todo!(),
            Comparison::GreaterThan => todo!(),
            Comparison::GreaterOrEquals => todo!(),
        }
    }
}