use crate::parser::op::Statement;

pub use self::environment::Environment;
use crate::parser::errors::ErrorCode;

pub mod expression;
pub mod prolog;
use crate::eval::prolog::*;

mod environment;
pub(crate) mod comparison;

pub(crate) use crate::values::{Object, Type, string_to_decimal, string_to_double, object_to_qname, atomization, sequence_atomization};

pub(crate) mod navigation;
mod arithmetic;
mod piping;
pub(crate) mod sequence_type;

pub(crate) mod helpers;
use helpers::*;
use crate::eval::expression::{Expression, NodeTest};
use crate::tree::Reference;


pub type ErrorInfo = (ErrorCode, String);
// pub type EvalResult = Result<(Box<Environment>, Iter<'a, Answer>), (ErrorCode, String)>;
// pub type EvalResult = Result<(Box<Environment>, Answer), (ErrorCode, String)>;
pub type EvalResult = Result<(Box<Environment>, Object), ErrorInfo>;

// initial_node_sequence
#[derive(Debug, Clone, PartialEq)]
pub enum INS {
    Root,
    RootDescendantOrSelf,
    DescendantOrSelf,
}

#[derive(Debug, Clone)]
pub struct DynamicContext {
    pub initial_node_sequence: Option<INS>,
    pub item: Object,
    pub position: Option<usize>,
    pub last: Option<usize>,
}

impl DynamicContext {
    pub(crate) fn nothing() -> Self {
        Self {
            initial_node_sequence: None,
            item: Object::Nothing,
            position: None,
            last: None,
        }
    }
}

pub(crate) fn eval_statements(statements: Vec<Statement>, env: Box<Environment>, context: &DynamicContext) -> EvalResult {

    let mut result = Object::Empty;

    let mut current_env = env;

    for statement in statements {
        let (new_env, new_result) = eval_statement(statement, current_env, context)?;
        current_env = new_env;

        result = new_result;

        if let &Object::Return(_) = &result {
            return Ok((current_env, result));
        }
    }

    Ok((current_env, result))
}

fn eval_statement(statement: Statement, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
    match statement {
        Statement::Prolog(exprs) => eval_prolog(exprs, env),
        Statement::Program(expr) => expr.eval(env, context),
    }
}

pub(crate) fn eval_prolog(exprs: Vec<Box<dyn Expression>>, env: Box<Environment>) -> EvalResult {
    let mut current_env = env;

    for expr in exprs {
        let (new_env, _) = expr.eval(current_env, &DynamicContext::nothing())?;
        current_env = new_env;
    }

    Ok((current_env, Object::Nothing))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Axis {
    // forward navigation
    ForwardSelf,
    ForwardAttribute,
    ForwardChild,
    ForwardDescendant,
    ForwardDescendantOrSelf,
    ForwardFollowing,
    ForwardFollowingSibling,

    // reverse navigation
    ReverseParent,
    ReverseAncestor,
    ReverseAncestorOrSelf,
    ReversePreceding,
    ReversePrecedingSibling,
}

fn step_and_test(step: &Axis, test: &Box<dyn NodeTest>, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
    match &context.item {
        Object::Nothing => Err((ErrorCode::XPDY0002, String::from("TODO"))),
        Object::Empty => Ok((env, Object::Empty)),
        Object::Node(rf) => {
            let mut result = vec![];
            step_and_test_for_node(step, test, rf, context, &mut result)?;
            // println!("RESULT {} {:#?}", result.len(), result);
            sort_and_dedup(&mut result);
            relax(env, result)
        },
        Object::Sequence(items) => {
            let mut result = vec![];
            for item in items {
                match item {
                    Object::Nothing => {
                        return Err((ErrorCode::XPDY0002, String::from("TODO")));
                    },
                    Object::Node(rf) => {
                        step_and_test_for_node(step, test, rf, context, &mut result)?;
                    }
                    _ => panic!()
                }
            }
            //println!("RESULT {} {:#?}", result.len(), result);
            sort_and_dedup(&mut result);
            relax(env, result)
        },
        _ => Err((ErrorCode::XPTY0019, String::from("TODO")))
    }
}

fn step_and_test_for_node<'a>(axis: &Axis, test: &Box<dyn NodeTest>, rf: &Reference, context: &DynamicContext, result: &mut Vec<Object>) -> Result<(), ErrorInfo> {
    match axis {
        Axis::ForwardSelf |
        Axis::ForwardChild |
        Axis::ForwardAttribute |
        Axis::ForwardDescendant |
        Axis::ForwardDescendantOrSelf => {
            for child in rf.forward(&context.initial_node_sequence, axis) {
                if test.test_node(&child) {
                    result.push(Object::Node(child))
                }
            }
        }
        Axis::ReverseParent => {
            if let Some(parent) = rf.parent() {
                if test.test_node(&parent) {
                    result.push(Object::Node(parent))
                }
            }
        }
        _ => todo!()
    }

    Ok(())
}


fn eval_predicates(exprs: &Vec<Predicate>, env: Box<Environment>, value: Object, context: &DynamicContext) -> EvalResult {
    let mut current_env = env;
    let mut result = value;

    for expr in exprs {
        let Predicate { expr: cond } = expr;

        let (new_env, new_value) = cond.predicate(current_env, context, result)?;
        current_env = new_env;
        result = new_value;
    }

    Ok((current_env, result))
}

pub struct RangeIterator {
    till: i128,
    next: i128,
    step: i128
}

impl RangeIterator {

    pub(crate) fn create(min: i128, max: i128) -> (Self, usize) {
        if min > max {
            (RangeIterator::new(min, -1, max), (min - max).min(0) as usize)
        } else {
            (RangeIterator::new(min, 1, max), (max - min).min(0) as usize)
        }
    }

    fn new(next: i128, step: i128, till: i128) -> Self {
        RangeIterator {
            till, next, step
        }
    }
}

impl Iterator for RangeIterator {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next;
        self.next = self.next + self.step;

        if (self.step > 0 && curr <= self.till) || (self.step < 0 && curr >= self.till) {
            Some(Object::Atomic(Type::Integer(curr)))
        } else {
            None
        }
    }
}

pub(crate) fn object_to_integer(env: &Box<Environment>, object: Object) -> Result<i128, ErrorInfo> {
    match object {
        Object::Atomic(t) => {
            match t {
                Type::Integer(num) => Ok(num),
                Type::Untyped(num) => {
                    match num.parse() {
                        Ok(v) => Ok(v),
                        Err(..) => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", num)))
                    }
                },
                _ => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", t)))
            }
        },
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(num) => {
                    match num.parse() {
                        Ok(v) => Ok(v),
                        Err(..) => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", num)))
                    }
                },
                Err(msg) => Err((ErrorCode::XPTY0004, format!("can't convert node to int")))
            }
        }
        _ => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", object)))
    }
}

// TODO: optimize!!!
pub(crate) fn object_to_iterator(object: &Object) -> Vec<Object> {
    match object {
        Object::Empty => Vec::new(),
        Object::Node(..) |
        Object::Atomic(..) => {
            let mut result = Vec::with_capacity(1);
            result.push(object.clone());
            result
        },
        Object::Range { min , max } => {
            let (it, count) = RangeIterator::create(*min, *max);

            let mut result = Vec::with_capacity(count.min(0) as usize);
            for item in it {
                result.push(item);
            }
            result
        },
        Object::Array(items) => {
            items.clone() // optimize?
        },
        Object::Sequence(items) => {
            items.clone() // optimize?
        },
        _ => panic!("TODO object_to_iterator {:?}", object)
    }
}

// TODO: optimize!!!
pub(crate) fn object_owned_to_sequence<'a>(object: Object) -> Vec<Object> {
    // println!("object_to_iterator for {:?}", object);
    match object {
        Object::Empty => vec![],
        Object::Range { .. } |
        Object::Node(..) |
        Object::Atomic(..) => {
            let mut result = Vec::with_capacity(1);
            result.push(object);
            result
        },
        // Object::Range { min , max } => {
        //     let (it, count) = RangeIterator::create(min, max);
        //     let mut result = Vec::with_capacity(count.min(0) as usize);
        //     for item in it {
        //         result.push(item);
        //     }
        //     result
        // },
        Object::Array(items) => {
            items
        },
        Object::Sequence(items) => {
            items
        },
        _ => panic!("TODO object_to_iterator {:?}", object)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::*;

    #[test]
    fn eval1() {
        test_eval(
            "empty(<a/>/a)",
            Object::Atomic(Type::Boolean(true))
        )
    }

    #[test]
    fn eval2() {
        test_eval(
            "deep-equal(string-to-codepoints($result),
            (97, 10, 10, 10, 32, 10, 115, 116, 114, 105, 110, 103, 32, 108, 105, 116, 101, 114, 97, 108, 32, 10))",
            Object::Empty
        )
    }

    fn test_eval(input: &str, expected: Object) {
        let result = parse(input);
        if result.is_ok() {
            let program = result.unwrap();

            let env = Environment::create();

            let (_, result) = eval_statements(program, env, &DynamicContext::nothing()).unwrap();

            assert_eq!(
                result,
                expected
            );
        } else {
            panic!("parse return error");
        }
    }
}
