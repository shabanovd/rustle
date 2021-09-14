use crate::eval::{Object, EvalResult, atomization, Type, NumberCase, string_to_double, Environment, object_to_iterator};
use crate::parser::op::Operator;
use core::ops;
use std::any::Any;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::parser::errors::ErrorCode;

// XS_DOUBLE    XS_DOUBLE       > DOUBLE_DOUBLE
// XS_DOUBLE    XS_FLOAT        > DOUBLE_FLOAT
// XS_DOUBLE    XS_DECIMAL      > DOUBLE_DECIMAL
// XS_DOUBLE    XS_INTEGER*     > DOUBLE_DECIMAL

// XS_FLOAT     XS_DOUBLE       > FLOAT_DOUBLE
// XS_FLOAT     XS_FLOAT        > FLOAT_FLOAT
// XS_FLOAT     XS_DECIMAL      > FLOAT_DECIMAL
// XS_FLOAT     XS_INTEGER*     > FLOAT_DECIMAL
// XS_DECIMAL   XS_DOUBLE       > DECIMAL_DOUBLE
// XS_DECIMAL   XS_FLOAT        > DECIMAL_FLOAT
// XS_DECIMAL   XS_DECIMAL      > DECIMAL_DECIMAL
// XS_DECIMAL   XS_INTEGER*     > DECIMAL_DECIMAL
// XS_DATE_TIME XS_DATE_TIME    > DATETIME_DATETIME
// XS_DATE_TIME XS_DURATION     > DATETIME_DURATION
// XS_DURATION  XS_DATE_TIME    > DURATION_DATETIME
// XS_DURATION  XS_DURATION     > DURATION_DURATION
// XS_DURATION  XS_DOUBLE       > DURATION_NUMERIC
// XS_DURATION  XS_FLOAT        > DURATION_NUMERIC
// XS_DURATION  XS_DECIMAL      > DURATION_NUMERIC
// XS_DURATION  XS_INTEGER      > DURATION_NUMERIC

// XS_DOUBLE    XS_DURATION     > NUMERIC_DURATION

// XS_FLOAT     XS_DURATION     > NUMERIC_DURATION
// XS_DECIMAL   XS_DURATION     > NUMERIC_DURATION
// XS_INTEGER*  XS_DURATION     > NUMERIC_DURATION

fn number_number(
    l: Number,
    r: Number,
    op: fn(Decimal, Decimal) -> (Option<Decimal>, NumberCase)
) -> (Option<Decimal>, NumberCase) {
    if let Some(l) = l.number {
        if let Some(r) = r.number {
            op(l, r)
        } else {
            (None, r.case)
        }
    } else {
        (None, l.case)
    }
}

trait Operand {
    fn add(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand>;
    fn sub(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand>;
    fn mul(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand>;
    fn div(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand>;

    fn create(&self, number: Option<Decimal>, case: NumberCase) -> Box<dyn Operand>;

    fn level(&self) -> u8;

    fn integer(&self) -> i128;
    fn is_integer(&self) -> bool;

    fn number(&self) -> Number;

    fn to_atomic(&self) -> Object;
}

struct Number {
    number: Option<Decimal>,
    case: NumberCase
}

struct VDouble {
    number: Option<Decimal>,
    case: NumberCase
}

impl Operand for VDouble {
    fn add(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l + r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn sub(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l - r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn mul(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l * r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn div(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l / r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn level(&self) -> u8 { 4 }

    fn create(&self, number: Option<Decimal>, case: NumberCase) -> Box<dyn Operand> {
        Box::new(VDouble { number, case })
    }

    fn integer(&self) -> i128 { panic!("internal error") }
    fn is_integer(&self) -> bool { false }

    fn number(&self) -> Number {
        Number { number: self.number, case: self.case.clone() }
    }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Double { number: self.number, case: self.case.clone() })
    }
}

struct VFloat {
    number: Option<Decimal>,
    case: NumberCase
}

impl Operand for VFloat {
    fn add(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l + r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn sub(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l - r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn mul(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l * r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn div(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l / r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn level(&self) -> u8 { 3 }

    fn create(&self, number: Option<Decimal>, case: NumberCase) -> Box<dyn Operand> {
        Box::new(VFloat { number, case })
    }

    fn integer(&self) -> i128 { panic!("internal error") }
    fn is_integer(&self) -> bool { false }

    fn number(&self) -> Number {
        Number { number: self.number, case: self.case.clone() }
    }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Float { number: self.number, case: self.case.clone() })
    }
}

struct VDecimal {
    number: Option<Decimal>,
    case: NumberCase
}

impl Operand for VDecimal {
    fn add(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l + r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn sub(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l - r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn mul(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l * r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn div(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
            (Some(l / r), NumberCase::Normal)
        });
        if self.level() > rhs.level() {
            self.create(number, case)
        } else {
            rhs.create(number, case)
        }
    }

    fn level(&self) -> u8 { 2 }

    fn create(&self, number: Option<Decimal>, case: NumberCase) -> Box<dyn Operand> {
        Box::new(VDecimal { number, case })
    }

    fn integer(&self) -> i128 { panic!("internal error") }
    fn is_integer(&self) -> bool { false }

    fn number(&self) -> Number {
        Number { number: self.number, case: self.case.clone() }
    }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Decimal { number: self.number, case: self.case.clone() })
    }
}

struct VInteger {
    number: i128
}

impl Operand for VInteger {
    fn add(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        if rhs.is_integer() {
            Box::new(VInteger { number: self.integer() + rhs.integer() })
        } else {
            let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
                (Some(l + r), NumberCase::Normal)
            });
            rhs.create(number, case)
        }
    }

    fn sub(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        if rhs.is_integer() {
            Box::new(VInteger { number: self.integer() - rhs.integer() })
        } else {
            let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
                (Some(l - r), NumberCase::Normal)
            });
            rhs.create(number, case)
        }
    }

    fn mul(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        if rhs.is_integer() {
            Box::new(VInteger { number: self.integer() * rhs.integer() })
        } else {
            let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
                (Some(l * r), NumberCase::Normal)
            });
            rhs.create(number, case)
        }
    }

    fn div(&self, rhs: Box<dyn Operand>) -> Box<dyn Operand> {
        if rhs.is_integer() {
            let number = self.integer() as f64 / rhs.integer() as f64;
            if let Some(number) = Decimal::from_f64(number) {
                Box::new(VDecimal { number: Some(number), case: NumberCase::Normal })
            } else {
                panic!("error")
            }
        } else {
            let (number, case) = number_number(self.number(), rhs.number(), |l, r| {
                (Some(l / r), NumberCase::Normal)
            });
            rhs.create(number, case)
        }
    }

    fn level(&self) -> u8 { 1 }

    fn create(&self, number: Option<Decimal>, case: NumberCase) -> Box<dyn Operand> {
        panic!("internal error")
    }

    fn integer(&self) -> i128 { self.number }
    fn is_integer(&self) -> bool { true }

    fn number(&self) -> Number {
        Number { number: Some(Decimal::from(self.number)), case: NumberCase::Normal }
    }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Integer(self.number))
    }
}

struct VDateTime {
}

struct VDuration {
}

fn object_to_items(object: &Object) -> Vec<Object> {
    match object {
        Object::ForBinding { name, values} => {
            object_to_iterator(values)
        },
        _ => vec![object.clone()]
    }
}

pub fn eval_arithmetic(env: Box<Environment>, operator: Operator, left: Object, right: Object) -> EvalResult {

    let mut current_env = env;
    let mut result = vec![];

    let it_left = object_to_items(&left);
    for l in it_left {

        let it_right = object_to_items(&right);
        for r in it_right {

            let (new_env, value) = eval_arithmetic_item(current_env, operator.clone(), l.clone(), r.clone())?;
            current_env = new_env;

            result.push(value);
        }
    }

    if result.len() == 0 {
        Ok((current_env, Object::Empty))
    } else if result.len() == 1 {
        let item = result.remove(0);
        Ok((current_env, item))
    } else {
        Ok((current_env, Object::Sequence(result)))
    }
}

pub fn eval_arithmetic_item(env: Box<Environment>, operator: Operator, left: Object, right: Object) -> EvalResult {
    // if DEBUG {
    println!("before atomization");
    println!("left_result {:?}", left);
    println!("right_result {:?}", right);
    // }

    let left = match atomization(left) {
        Ok(v) => v,
        Err(e) => return Err((e, String::from("TODO")))
    };
    let right = match atomization(right) {
        Ok(v) => v,
        Err(e) => return Err((e, String::from("TODO")))
    };

    println!("after atomization");
    println!("left_result {:?}", left);
    println!("right_result {:?}", right);

    let left_value = match into_type(left) {
        Ok(v) => v,
        Err(e) => return Err((e, String::from("TODO")))
    };
    let right_value = match into_type(right) {
        Ok(v) => v,
        Err(e) => return Err((e, String::from("TODO")))
    };

    // println!("after into_type");
    // println!("left_result {:?}", left_value);
    // println!("right_result {:?}", right_value);

    let result = match operator {
        // Operator::Unknown => {}
        Operator::Plus => {
            left_value.add(right_value)
        }
        Operator::Minus => {
            left_value.sub(right_value)
        }
        Operator::Multiply => {
            left_value.mul(right_value)
        }
        Operator::Divide => {
            left_value.div(right_value)
        }
        // Operator::IDivide => {}
        // Operator::Mod => {}
        // Operator::GeneralEquals => {}
        // Operator::GeneralNotEquals => {}
        // Operator::GeneralLessThan => {}
        // Operator::GeneralLessOrEquals => {}
        // Operator::GeneralGreaterThan => {}
        // Operator::GeneralGreaterOrEquals => {}
        // Operator::ValueEquals => {}
        // Operator::ValueNotEquals => {}
        // Operator::ValueLessThan => {}
        // Operator::ValueLessOrEquals => {}
        // Operator::ValueGreaterThan => {}
        // Operator::ValueGreaterOrEquals => {}
        _ => panic!("error")
    };

    Ok((env, result.to_atomic()))
}

fn into_type(obj: Object) -> Result<Box<Operand>, ErrorCode> {
    match obj {
        Object::Atomic(t) => {
            match t {
                Type::Untyped(str) => {
                    match string_to_double(&str) {
                        Ok(Object::Atomic(Type::Double { number, case })) => {
                            Ok(Box::new(VDouble { number, case }))
                        },
                        _ => panic!("error")
                    }
                }
                // Type::dateTime() => {}
                // Type::dateTimeStamp() => {}
                // Type::Date { .. } => {}
                // Type::Time { .. } => {}
                // Type::Duration { .. } => {}
                // Type::YearMonthDuration { .. } => {}
                // Type::DayTimeDuration { .. } => {}
                Type::Integer(number) => Ok(Box::new( VInteger { number } )),
                Type::Decimal { number, case } => Ok(Box::new( VDecimal { number, case } )),
                Type::Float { number, case } => Ok(Box::new( VFloat { number, case } )),
                Type::Double { number, case } => Ok(Box::new( VDouble { number, case } )),
                // Type::nonPositiveInteger() => {}
                // Type::negativeInteger() => {}
                // Type::long() => {}
                // Type::int() => {}
                // Type::short() => {}
                // Type::byte() => {}
                // Type::nonNegativeInteger() => {}
                // Type::unsignedLong() => {}
                // Type::unsignedInt() => {}
                // Type::unsignedShort() => {}
                // Type::unsignedByte() => {}
                // Type::positiveInteger() => {}
                // Type::gYearMonth() => {}
                // Type::gYear() => {}
                // Type::gMonthDay() => {}
                // Type::gDay() => {}
                // Type::gMonth() => {}
                // Type::String(_) => {}
                // Type::NormalizedString(_) => {}
                // Type::Token(_) => {}
                // Type::language(_) => {}
                // Type::NMTOKEN(_) => {}
                // Type::Name(_) => {}
                // Type::NCName(_) => {}
                // Type::ID(_) => {}
                // Type::IDREF(_) => {}
                // Type::ENTITY(_) => {}
                // Type::Boolean(_) => {}
                // Type::base64Binary() => {}
                // Type::hexBinary() => {}
                // Type::AnyURI(_) => {}
                // Type::QName() => {}
                // Type::NOTATION() => {}
                _ => Err(ErrorCode::XPTY0004)
            }
        },
        _ => Err(ErrorCode::XPTY0004)
    }
}