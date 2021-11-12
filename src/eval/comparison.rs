use crate::eval::{Object, Type, Environment, EvalResult, atomization, relax, sequence_atomization, ErrorInfo};
use crate::serialization::object_to_string;
use crate::parser::parse_duration::string_to_dt_duration;
use std::cmp::Ordering;
use crate::parser::op::{Comparison, OperatorComparison};
use crate::eval::arithmetic::object_to_items;
use ordered_float::OrderedFloat;
use crate::parser::errors::ErrorCode;
use crate::values::{atomization_of_vec, QName, Types};
use crate::fns::object_to_bool;
use crate::tree::Reference;

type ObjectInEnv<'a> = (&'a Box<Environment>, Object);
type ObjectRefInEnv<'a> = (&'a Box<Environment>, &'a Object);
type NodeRefInEnv<'a> = (&'a Box<Environment>, &'a Reference);

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
                            let obj = comparison_of_items(
                                &operator,
                                (&current_env, &l_obj), (&current_env, &r_obj)
                            )?;
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
                        Ok(v) => Ok((env, object_to_bool(&v)?)),
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
) -> Result<Object, ErrorInfo> {
    match operator {
        OperatorComparison::GeneralEquals |
        OperatorComparison::GeneralNotEquals |
        OperatorComparison::GeneralLessThan |
        OperatorComparison::GeneralLessOrEquals |
        OperatorComparison::GeneralGreaterThan |
        OperatorComparison::GeneralGreaterOrEquals => {
            let value = general_comparison(operator, left, right)?;
            Ok(Object::Atomic(Type::Boolean(value)))
        },
        OperatorComparison::ValueEquals |
        OperatorComparison::ValueNotEquals |
        OperatorComparison::ValueLessThan |
        OperatorComparison::ValueLessOrEquals |
        OperatorComparison::ValueGreaterThan |
        OperatorComparison::ValueGreaterOrEquals => value_comparison(operator, left, right),
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

impl From<Ordering> for ValueOrdering {
    fn from(v: Ordering) -> Self {
        match v {
            Ordering::Less => ValueOrdering::Less,
            Ordering::Equal => ValueOrdering::Equal,
            Ordering::Greater => ValueOrdering::Greater,
        }
    }
}

pub(crate) fn eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueEquals, left, right)?;
    object_to_bool(&result)
}

pub(crate) fn ne(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueNotEquals, left, right)?;
    object_to_bool(&result)
}

pub(crate) fn ls_or_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueLessOrEquals, left, right)?;
    object_to_bool(&result)
}

pub(crate) fn ls(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueLessThan, left, right)?;
    object_to_bool(&result)
}

pub(crate) fn gr_or_eq(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueGreaterOrEquals, left, right)?;
    object_to_bool(&result)
}

pub(crate) fn gr(left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    let result = value_comparison(&OperatorComparison::ValueGreaterThan, left, right)?;
    object_to_bool(&result)
}

fn is_untyped(value: &Type) -> bool {
    match value {
        Type::Untyped(..) => true,
        _ => false
    }
}

fn is_numeric(value: &Type) -> bool {
    match value {
        Type::Integer(..) |
        Type::Decimal(..) |
        Type::Float(..) |
        Type::Double(..) => true,
        _ => false
    }
}

pub(crate) fn value_comparison_for_types(op: Comparison, left: &Type, right: &Type) -> Result<bool, ErrorInfo> {
    let cmp_result = left.value_comparison(right)?;
    op.is_it(cmp_result)
}

fn general_comparison_for_types(op: &OperatorComparison, left: &Type, right: &Type) -> Result<bool, ErrorInfo> {
    let is_untyped_left = is_untyped(left);
    let is_untyped_right = is_untyped(right);
    if is_untyped_left != is_untyped_right {
        return if is_untyped_left {
            let l = if is_numeric(right) {
                left.convert(Types::Double)?
            } else {
                left.convert(right.to_type())?
            };
            value_comparison_for_types(op.to_comparison(), &l, right)
        } else {
            let r = if is_numeric(left) {
                right.convert(Types::Double)?
            } else {
                right.convert(left.to_type())?
            };
            value_comparison_for_types(op.to_comparison(), left, &r)
        }
    } else {
        value_comparison_for_types(op.to_comparison(), left, right)
    }
}

fn type_in_range(t: &Type, min: &i128, max: &i128) -> Result<bool, ErrorInfo> {
    if let Some(num) = t.to_i128(false).as_ref() {
        if min <= max {
            Ok(num >= min && num <= max)
        } else {
            Ok(num >= max && num <= min)
        }
    } else {
        Ok(false)
    }
}

fn value_comparison(op: &OperatorComparison, left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<Object, ErrorInfo> {
    println!("value_comparison: {:?} vs {:?}", left.1, right.1);

    if left.1 == &Object::Empty {
        return Ok(Object::Empty);
    } else if left.1 == right.1 {
        match op.to_comparison() {
            Comparison::GreaterOrEquals |
            Comparison::LessOrEquals |
            Comparison::Equals => return Ok(Object::Atomic(Type::Boolean(true))),
            Comparison::NotEquals |
            Comparison::LessThan |
            Comparison::GreaterThan => {},
        }
    }
    match left.1 {
        Object::Empty => Ok(Object::Empty),
        Object::Atomic(lt) => {
            match right.1 {
                Object::Empty => Ok(Object::Empty),
                Object::Atomic(rt) => {
                    let cmp_result = lt.value_comparison(rt)?;
                    let value = op.to_comparison().is_it(cmp_result)?;
                    Ok(Object::Atomic(Type::Boolean(value)))
                },
                _ => todo!()
            }
        }
        _ => todo!()
    }
}

pub(crate) fn general_comparison(op: &OperatorComparison, left: ObjectRefInEnv, right: ObjectRefInEnv) -> Result<bool, ErrorInfo> {
    println!("general_comparison: {:?} vs {:?}", left.1, right.1);
    match left.1 {
        Object::Empty => Ok(false),
        Object::Atomic(lt) => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Atomic(rt) => {
                    general_comparison_for_types(op, lt, rt)
                }
                Object::Range { min, max } => {
                    type_in_range(lt, min, max)
                }
                Object::Node(r_rf) => {
                    let rv = match r_rf.to_typed_value() {
                        Ok(data) => Type::Untyped(data),
                        Err(msg) => return Err((ErrorCode::TODO, msg))
                    };
                    general_comparison_for_types(op, lt, &rv)
                }
                Object::Sequence(items) => {
                    for item in items {
                        match item {
                            Object::Atomic(rt) => {
                                if general_comparison_for_types(op, lt, rt)? {
                                    return Ok(true);
                                }
                            },
                            Object::Node(rf) => {
                                let rv = match rf.to_typed_value() {
                                    Ok(data) => Type::Untyped(data),
                                    Err(msg) => return Err((ErrorCode::TODO, msg))
                                };
                                if general_comparison_for_types(op, lt, &rv)? {
                                    return Ok(true);
                                }
                            },
                            _ => {} // ignore
                        }
                    }
                    Ok(false)
                }
                _ => panic!("{:?} vs {:?}", left.1, right.1) // Err((ErrorCode::XPTY0004, String::from("TODO")))
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
                Object::Atomic(rt) => {
                    type_in_range(rt, l_min, l_max)
                }
                _ => panic!("{:?} vs {:?}", left.1, right.1) // Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        }
        Object::Array(left_items) |
        Object::Sequence(left_items) => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Atomic(..) => {
                    for lo in left_items {
                        if general_comparison(op, (left.0, lo), right)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
                Object::Array(right_items) |
                Object::Sequence(right_items) => {
                    for left_item in left_items {
                        for right_item in right_items {
                            if general_comparison(op, (left.0, left_item), (right.0, &right_item))? {
                                return Ok(true);
                            }
                        }
                    }
                    Ok(false)
                }
                _ => panic!("{:?} vs {:?}", left.1, right.1) // Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        },
        Object::Node(l_rf) => {
            match right.1 {
                Object::Empty => Ok(false),
                Object::Atomic(rt) => {
                    match l_rf.to_typed_value() {
                        Ok(l_str) => general_comparison_for_types(op, &Type::Untyped(l_str), rt),
                        Err(msg) => Err((ErrorCode::XPTY0004, msg))
                    }
                }
                Object::Node(r_rf) => {
                    match l_rf.cmp(r_rf) {
                        Ordering::Equal => {
                            let cmp_result = ValueOrdering::from(Ordering::Equal);
                            op.to_comparison().is_it(cmp_result)
                        },
                        _ => {
                            let lv = match l_rf.to_typed_value() {
                                Ok(data) => Type::Untyped(data),
                                Err(msg) => return Err((ErrorCode::TODO, msg))
                            };
                            let rv = match r_rf.to_typed_value() {
                                Ok(data) => Type::Untyped(data),
                                Err(msg) => return Err((ErrorCode::TODO, msg))
                            };
                            general_comparison_for_types(op, &lv, &rv)
                        }
                    }
                }
                _ => panic!("{:?} vs {:?}", left.1, right.1) // Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        }
        _ => panic!("{:?} vs {:?}", left.1, right.1) // Err((ErrorCode::XPTY0004, String::from("TODO")))
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

pub(crate) fn deep_eq_sequence_and_range<'a>(env: &'a Box<Environment>, left_items: &'a Vec<Object>, min: i128, max: i128) -> Result<bool, ErrorInfo> {
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