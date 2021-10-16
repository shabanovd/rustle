use crate::eval::{Object, Type, Environment, EvalResult, atomization, relax, sequence_atomization, ErrorInfo};
use crate::serialization::object_to_string;
use crate::parser::parse_duration::string_to_dt_duration;
use std::cmp::Ordering;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::parser::op::OperatorComparison;
use crate::eval::arithmetic::object_to_items;
use ordered_float::OrderedFloat;
use crate::parser::errors::ErrorCode;
use crate::values::QName;
use crate::fns::object_to_bool;
use crate::tree::Reference;

type ObjectInEnv<'a, 'b> = (&'a Box<Environment<'b>>, Object);
type ObjectRefInEnv<'a, 'b> = (&'a Box<Environment<'b>>, &'a Object);
type NodeRefInEnv<'a, 'b> = (&'a Box<Environment<'b>>, &'a Reference);

// TODO: join with eval_arithmetic ?
pub fn eval_comparison(env: Box<Environment>, operator: OperatorComparison, left: Object, right: Object) -> EvalResult {

    let mut current_env = env;
    let mut result = vec![];

    let check_type = to_type(&operator);

    let it_left = object_to_items(&left);
    for l in it_left {

        match object_atomization(&current_env, &check_type, l) {
            Err(e) => return Err(e),
            Ok((None, None, Some(result))) => return Ok((current_env, result)),
            Ok((Some(l_obj), None, None)) => {

                let it_right = object_to_items(&right);
                for r in it_right {

                    match object_atomization(&current_env, &check_type, r) {
                        Err(e) => return Err(e),
                        Ok((None, None, Some(result))) => return Ok((current_env, result)),
                        Ok((Some(r_obj), None, None)) => {
                            let v = comparison_of_items(
                                &operator,
                                (&current_env, &l_obj), (&current_env, &r_obj)
                            )?;

                            let obj = Object::Atomic(Type::Boolean(v));
                            result.push(obj);
                        },
                        _ => panic!("internal error")
                    }
                }

            },
            Ok((None, Some(l_node), None)) => {

                let it_right = object_to_items(&right);
                for r in it_right {

                    match object_atomization(&current_env, &check_type, r) {
                        Err(e) => return Err(e),
                        Ok((None, None, Some(result))) => return Ok((current_env, result)),
                        Ok((None, Some(r_node), None)) => {
                            let v = comparison_of_nodes(
                                &operator,
                                (&current_env, &l_node), (&current_env, &r_node)
                            )?;

                            let obj = Object::Atomic(Type::Boolean(v));
                            result.push(obj);
                        }
                        _ => panic!("internal error")
                    }
                }

            },
            _ => panic!("internal error")
        };
    }

    relax(current_env, result)
}

pub fn eval_comparison_item(
    env: Box<Environment>,
    operator: OperatorComparison,
    left: Object,
    right: Object,
) -> Result<(Box<Environment>, bool), ErrorInfo> {
    let check_type = to_type(&operator);

    match object_atomization(&env, &check_type, left) {
        Err(e) => Err(e),
        Ok((None, None, Some(result))) => {
            let v = object_to_bool(&result)?;
            Ok((env, v))
        },
        Ok((Some(l_obj), None, None)) => {
            match object_atomization(&env, &check_type, right) {
                Err(e) => Err(e),
                Ok((None, None, Some(result))) => {
                    let v = object_to_bool(&result)?;
                    Ok((env, v))
                }
                Ok((Some(r_obj), None, None)) => {
                    match comparison_of_items(&operator, (&env, &l_obj), (&env, &r_obj)) {
                        Ok(v) => Ok((env, v)),
                        Err(e) => Err(e)
                    }
                },
                _ => panic!("internal error")
            }
        },
        Ok((None, Some(l_node), None)) => {
            match object_atomization(&env, &check_type, right) {
                Err(e) => Err(e),
                Ok((None, None, Some(result))) => {
                    let v = object_to_bool(&result)?;
                    Ok((env, v))
                }
                Ok((None, Some(r_node), None)) => {
                    match comparison_of_nodes(&operator, (&env, &l_node), (&env, &r_node)) {
                        Ok(v) => Ok((env, v)),
                        Err(e) => Err(e)
                    }
                },
                _ => panic!("internal error")
            }
        }
        _ => panic!("internal error")
    }
}

fn object_atomization<'a>(
    env: &'a Box<Environment>, checks_type: &'a ComparisonType, obj: Object
) -> Result<(Option<Object>, Option<Reference>, Option<Object>), ErrorInfo> {
    match checks_type {
        ComparisonType::Value => {
            match atomization(env, obj) {
                Ok(v) => Ok((Some(v), None, None)),
                Err(e) => Err(e)
            }
        },
        ComparisonType::General => {
            match sequence_atomization(env, obj) {
                Ok(v) => Ok((Some(v), None, None)),
                Err(e) => Err(e)
            }
        },
        ComparisonType::Node => {
            match obj {
                Object::Empty => Ok((None, None, Some(Object::Empty))),
                Object::Node(rf) => Ok((None, Some(rf), None)),
                _ => Err((ErrorCode::TODO, String::from("TODO")))
            }
        }
    }
}

enum ComparisonType {
    General,
    Value,
    Node,
}

fn to_type(operator: &OperatorComparison) -> ComparisonType {
    match operator {
        OperatorComparison::GeneralEquals |
        OperatorComparison::GeneralNotEquals |
        OperatorComparison::GeneralLessThan |
        OperatorComparison::GeneralLessOrEquals |
        OperatorComparison::GeneralGreaterThan |
        OperatorComparison::GeneralGreaterOrEquals => ComparisonType::General,
        OperatorComparison::ValueEquals |
        OperatorComparison::ValueNotEquals |
        OperatorComparison::ValueLessThan |
        OperatorComparison::ValueLessOrEquals |
        OperatorComparison::ValueGreaterThan |
        OperatorComparison::ValueGreaterOrEquals => ComparisonType::Value,
        OperatorComparison::NodeIs |
        OperatorComparison::NodePrecedes |
        OperatorComparison::NodeFollows => ComparisonType::Node,
    }
}

fn comparison_of_nodes(
    operator: &OperatorComparison,
    left: NodeRefInEnv,
    right: NodeRefInEnv,
) -> Result<bool, ErrorInfo> {
    match operator {
        OperatorComparison::NodeIs => node_is(left, right),
        OperatorComparison::NodePrecedes => node_precedes(left, right),
        OperatorComparison::NodeFollows => node_follows(left, right),
        _ => panic!("internal error")
    }
}

pub fn comparison_of_items(
    operator: &OperatorComparison,
    left: ObjectRefInEnv,
    right: ObjectRefInEnv,
) -> Result<bool, ErrorInfo> {
    match operator {
        OperatorComparison::GeneralEquals => general_eq(left, right),
        OperatorComparison::GeneralNotEquals => todo!(),
        OperatorComparison::GeneralLessThan => todo!(),
        OperatorComparison::GeneralLessOrEquals => todo!(),
        OperatorComparison::GeneralGreaterThan => todo!(),
        OperatorComparison::GeneralGreaterOrEquals => todo!(),
        OperatorComparison::ValueEquals => eq(left, right),
        OperatorComparison::ValueNotEquals => ne(left, right),
        OperatorComparison::ValueLessThan => ls(left, right),
        OperatorComparison::ValueLessOrEquals => ls_or_eq(left, right),
        OperatorComparison::ValueGreaterThan => gr(left, right),
        OperatorComparison::ValueGreaterOrEquals => gr_or_eq(left, right),
        _ => panic!("internal error")
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ValueOrdering {
    Less,
    Equal,
    QNameEqual,
    QNameNotEqual,
    AlwaysNotEqual,
    Greater,
}

impl ValueOrdering {
    fn from(v: Ordering) -> Option<Self> {
        match v {
            Ordering::Less => Some(ValueOrdering::Less),
            Ordering::Equal => Some(ValueOrdering::Equal),
            Ordering::Greater => Some(ValueOrdering::Greater),
        }
    }
}

fn cmp(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Option<ValueOrdering> {
    match left.1 {
        Object::Atomic(Type::Untyped(..)) |
        Object::Atomic(Type::AnyURI(..)) |
        Object::Atomic(Type::String(..)) |
        Object::Atomic(Type::NormalizedString(..)) |
        Object::CharRef {..} |
        Object::EntityRef(..) => {
            if left.1 == right.1 {
                return Some(ValueOrdering::Equal);
            }
            // xs:string or xs:anyURI => xs:string
            if let Some(l_str) = object_to_string_if_string(left.0, left.1) {
                if let Some(r_str) = object_to_string_if_string(right.0, right.1) {
                    return ValueOrdering::from(l_str.cmp(&r_str));
                }
            }
            None
        }
        Object::Atomic(Type::QName {..}) => {
            if left.1 == right.1 {
                return Some(ValueOrdering::QNameEqual);
            }
            if let Some(l_qname) = object_to_qname_if_qname(left.1) {
                if let Some(r_qname) = object_to_qname_if_qname(right.1) {
                    return match l_qname.partial_cmp(&r_qname) {
                        Some(Ordering::Equal) => Some(ValueOrdering::QNameEqual),
                        _ => Some(ValueOrdering::QNameNotEqual),
                    }
                }
            }
            None
        },
        Object::Atomic(Type::Boolean(lbt)) => {
            let rbt = match object_to_bool(right.1) {
                Ok(v) => v,
                Err(e) => {
                    return None;
                }
            };
            return ValueOrdering::from(lbt.cmp(&rbt));
        },
        Object::Atomic(Type::Integer(..)) |
        Object::Atomic(Type::Decimal(..)) |
        Object::Atomic(Type::Float(..)) |
        Object::Atomic(Type::Double(..)) => {
            let lnt = object_to_number_type(left.1);
            let rnt = object_to_number_type(right.1);

            // xs:integer, xs:decimal or xs:float => xs:float
            // xs:integer, xs:decimal, xs:float, or xs:double => xs:double
            if let Some(lnt) = lnt {
                if let Some(rnt) = rnt {
                    let nt = if lnt > rnt { lnt } else { rnt };
                    match nt {
                        NT::Integer => {
                            if let Some(left_num) = object_to_i128(left.1) {
                                if let Some(right_num) = object_to_i128(right.1) {
                                    return ValueOrdering::from(left_num.cmp(&right_num));
                                }
                            }
                            return None;
                        },
                        NT::Decimal => {
                            if let Some(left_num) = object_to_decimal(left.1) {
                                if let Some(right_num) = object_to_decimal(right.1) {
                                    return ValueOrdering::from(left_num.cmp(&right_num));
                                }
                            }
                            return None;
                        },
                        NT::Float => {
                            if let Some(left_num) = object_to_float(left.1) {
                                if let Some(right_num) = object_to_float(right.1) {
                                    return if left_num.is_nan() || right_num.is_nan() {
                                        Some(ValueOrdering::AlwaysNotEqual)
                                    } else {
                                        ValueOrdering::from(left_num.cmp(&right_num))
                                    };
                                }
                            }
                            return None;
                        },
                        NT::Double => {
                            if let Some(left_num) = object_to_double(left.1) {
                                if let Some(right_num) = object_to_double(right.1) {
                                    return if left_num.is_nan() || right_num.is_nan() {
                                        Some(ValueOrdering::AlwaysNotEqual)
                                    } else {
                                        ValueOrdering::from(left_num.cmp(&right_num))
                                    };
                                }
                            }
                            return None;
                        },
                    }
                }
            }
            None
        },
        _ => None
    }
}

pub(crate) fn eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::Equal) |
        Some(ValueOrdering::QNameEqual) => Ok(true),
        Some(..) => Ok(false),
        None => Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

pub(crate) fn ne(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::Less) |
        Some(ValueOrdering::Greater) |
        Some(ValueOrdering::AlwaysNotEqual) |
        Some(ValueOrdering::QNameNotEqual) => Ok(true),
        Some(..) => Ok(false),
        None => Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

pub(crate) fn ls_or_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::QNameEqual) |
        Some(ValueOrdering::QNameNotEqual) |
        None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
        Some(ValueOrdering::Equal) |
        Some(ValueOrdering::Less) => Ok(true),
        Some(..) => Ok(false),
    }
}

pub(crate) fn ls(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::QNameEqual) |
        Some(ValueOrdering::QNameNotEqual) |
        None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
        Some(ValueOrdering::Less) => Ok(true),
        Some(..) => Ok(false),
    }
}

pub(crate) fn gr_or_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::QNameEqual) |
        Some(ValueOrdering::QNameNotEqual) |
        None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
        Some(ValueOrdering::Equal) |
        Some(ValueOrdering::Greater) => Ok(true),
        Some(..) => Ok(false),
    }
}

pub(crate) fn gr(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match cmp(left, right) {
        Some(ValueOrdering::QNameEqual) |
        Some(ValueOrdering::QNameNotEqual) |
        None => Err((ErrorCode::XPTY0004, String::from("TODO"))),
        Some(v) => Ok(v == ValueOrdering::Greater),
    }
}

pub(crate) fn general_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    match left.1 {
        Object::Empty => Ok(false),
        Object::Atomic(lt) => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Atomic(Type::Untyped(rs)) => {
                    match lt {
                        Type::Untyped(ls) => {
                            Ok(ls == rs)
                        }
                        Type::DayTimeDuration { .. } => {
                            match string_to_dt_duration(rs) {
                                Ok(rd) => Ok(lt == &rd),
                                Err(..) => Err((ErrorCode::XPTY0004, String::from("TODO")))
                            }
                        }
                        Type::Integer(..) |
                        Type::Decimal {..} |
                        Type::Float {..} |
                        Type::Double {..} => {
                            if let Ok(number) = rs.parse() {
                                let rv = Object::Atomic(Type::Double(number));
                                eq(left, (right.0, &rv))
                            } else {
                                Err((ErrorCode::XPTY0004, String::from("TODO")))
                            }
                        }
                        _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                    }
                }
                Object::Atomic(..) => {
                    eq(left, right)
                },
                Object::Range { min, max } => {
                    match lt {
                        Type::Integer(ls) => {
                            if min <= max {
                                Ok(ls >= min && ls <= max)
                            } else {
                                Ok(ls >= max && ls <= min)
                            }
                        },
                        _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                    }
                }
                Object::Sequence(items) => {
                    for item in items {
                        if eq(left, (right.0, item))? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        }
        Object::Range { min: l_min, max: l_max} => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Range { min: r_min, max: r_max} => {
                    let (l_min, l_max) = if l_min <= l_max {
                        (l_min, l_max)
                    } else {
                        (l_max, l_min)
                    };

                    let (r_min, r_max) = if r_min <= r_max {
                        (r_min, r_max)
                    } else {
                        (r_max, r_min)
                    };

                    Ok((l_min <= r_max) && (l_max >= r_min))
                },
                _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        }
        Object::Sequence(left_items) => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Atomic(..) => {
                    for item in left_items {
                        if eq((left.0, item), right)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
                Object::Sequence(right_items) => {
                    for left_item in left_items {
                        for right_item in right_items {
                            if eq((left.0, left_item), (right.0, &right_item))? {
                                return Ok(true);
                            }
                        }
                    }
                    Ok(false)
                }
                _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        },
        _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

pub(crate) fn deep_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    if left.1 == right.1 {
        Ok(true)
    } else {
        match left.1 {
            Object::Atomic(..) => {
                match right.1 {
                    Object::Atomic(..) => {
                        eq(left, right)
                    }
                    _ => Ok(false)
                }
            }
            Object::Range { min: left_min, max: left_max } => {
                match right.1 {
                    Object::Range { min: right_min, max: right_max } => {
                        Ok(left_min == right_min && left_max == right_max)
                    },
                    Object::Sequence(right_items) => {
                        deep_eq_sequence_and_range(right.0, right_items, *left_min, *left_max)
                    }
                    _ => panic!("TODO {:?}", right.1)
                }
            },
            Object::Sequence(left_items) => {
                match right.1 {
                    Object::Range { min, max } => {
                        deep_eq_sequence_and_range(left.0, left_items, *min, *max)
                    },
                    Object::Sequence(right_items) => {
                        if left_items.len() != right_items.len() {
                            Ok(false)
                        } else {
                            let mut left_it = left_items.iter();
                            let mut right_it = right_items.iter();

                            loop {
                                if let Some(left_item) = left_it.next() {
                                    if let Some(right_item) = right_it.next() {
                                        if deep_eq((&left.0, left_item), (&right.0, right_item))? {
                                            return Ok(false);
                                        }
                                    } else {
                                        return Ok(false);
                                    }
                                } else {
                                    if let Some(..) = right_it.next() {
                                        return Ok(false);
                                    }
                                    return Ok(true);
                                }
                            }
                        }
                    },
                    _ => Ok(false)
                }
            },
            Object::Node(l_rf) => {
                match right.1 {
                    Object::Node(r_rf) => {
                        match l_rf.cmp(r_rf) {
                            Ordering::Equal => {
                                Ok(true)
                            }
                            _ => Ok(false)
                        }
                    }
                    _ => Ok(false)
                }
            }
            _ => panic!("TODO {:?}", left.1)
        }
    }
}

pub(crate) fn deep_eq_sequence_and_range<'a, 'b>(env: &'a Box<Environment<'b>>, left_items: &'a Vec<Object>, min: i128, max: i128) -> Result<bool, ErrorInfo> {
    let (min, max) = if min <= max {
        (min, max)
    } else {
        (max, min)
    };

    if left_items.len() != ((max - min).abs() + 1).max(0) as usize {
        Ok(false)
    } else {
        let mut left_it = left_items.iter();

        let mut right_item = min;
        let step = 1;

        loop {
            if let Some(left_item) = left_it.next() {
                if deep_eq((env, left_item), (env, &Object::Atomic(Type::Integer(right_item))))? {
                    return Ok(false);
                }

                right_item += step;
            } else {
                return Ok((right_item - step) == max);
            }
        }
    }
}

pub(crate) fn node_is(left: NodeRefInEnv, right: NodeRefInEnv) -> Result<bool, ErrorInfo> {
    todo!()
}

pub(crate) fn node_precedes(left: NodeRefInEnv, right: NodeRefInEnv) -> Result<bool, ErrorInfo> {
    todo!()
}

pub(crate) fn node_follows(left: NodeRefInEnv, right: NodeRefInEnv) -> Result<bool, ErrorInfo> {
    todo!()
}

#[derive(Eq, PartialEq, PartialOrd)]
pub enum NT {
    Integer = 1,
    Decimal = 2,
    Float   = 3,
    Double  = 4,
}

fn object_to_number_type(obj: &Object) -> Option<NT> {
    match obj {
        Object::Atomic(Type::Integer(..)) => Some(NT::Integer),
        Object::Atomic(Type::Decimal(..)) => Some(NT::Decimal),
        Object::Atomic(Type::Float(..)) => Some(NT::Float),
        Object::Atomic(Type::Double(..)) => Some(NT::Double),
        _ => None
    }
}

fn object_to_i128(obj: &Object) -> Option<i128> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(*num),
        // Object::Atomic(Type::Decimal(num)) => Some(num),
        // Object::Atomic(Type::Double(num)) => Some(num),
        _ => None
    }
}

fn object_to_decimal(obj: &Object) -> Option<BigDecimal> {
    match obj {
        Object::Atomic(Type::Integer(number)) => BigDecimal::from_i128(*number),
        Object::Atomic(Type::Decimal(number)) => Some(number.clone()),
        Object::Atomic(Type::Float(number)) => {
            BigDecimal::from_f32(number.into_inner())
        }
        Object::Atomic(Type::Double(number)) => {
            BigDecimal::from_f64(number.into_inner())
        },
        _ => None
    }
}

fn object_to_float(obj: &Object) -> Option<OrderedFloat<f32>> {
    match obj {
        Object::Atomic(Type::Integer(number)) => OrderedFloat::from_i128(*number),
        Object::Atomic(Type::Decimal(number)) => {
            if let Some(number) = number.to_f32() {
                OrderedFloat::from_f32(number)
            } else {
                None
            }
        },
        Object::Atomic(Type::Float(number)) => Some(*number),
        Object::Atomic(Type::Double(number)) => {
            if let Some(number) = number.to_f32() {
                OrderedFloat::from_f32(number)
            } else {
                None
            }
        },
        _ => None
    }
}

fn object_to_double(obj: &Object) -> Option<OrderedFloat<f64>> {
    match obj {
        Object::Atomic(Type::Integer(number)) => OrderedFloat::from_i128(*number),
        Object::Atomic(Type::Decimal(number)) => {
            if let Some(number) = number.to_f64() {
                OrderedFloat::from_f64(number)
            } else {
                None
            }
        },
        Object::Atomic(Type::Float(number)) => {
            if let Some(number) = number.to_f64() {
                OrderedFloat::from_f64(number)
            } else {
                None
            }
        },
        Object::Atomic(Type::Double(number)) => Some(number.clone()),
        _ => None
    }
}

fn object_to_string_if_string(env: &Box<Environment>, obj: &Object) -> Option<String> {
    match obj {
        Object::Atomic(Type::Untyped(..)) |
        Object::Atomic(Type::AnyURI(..)) |
        Object::Atomic(Type::String(..)) |
        Object::Atomic(Type::NormalizedString(..)) |
        Object::CharRef {..} |
        Object::EntityRef(..) => {
            Some(object_to_string(env, obj))
        }
        _ => None
    }
}

fn object_to_qname_if_qname(obj: &Object) -> Option<QName> {
    match obj {
        Object::Atomic(Type::QName { url, prefix, local_part }) => {
            Some(QName { url: url.clone(), prefix: prefix.clone(), local_part: local_part.clone() })
        }
        _ => None
    }
}