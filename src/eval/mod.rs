use crate::parser::op::Statement;

pub use self::environment::Environment;
use crate::parser::errors::ErrorCode;

pub mod expression;
pub mod prolog;
use crate::eval::prolog::*;

mod environment;
pub(crate) mod comparison;
mod value;
pub(crate) use value::*;

mod arithmetic;
mod piping;

pub(crate) mod helpers;
use helpers::*;
use crate::eval::expression::Expression;

// pub type EvalResult<'a> = Result<(Box<Environment<'a>>, Iter<'a, Answer>), (ErrorCode, String)>;
// pub type EvalResult<'a> = Result<(Box<Environment<'a>>, Answer), (ErrorCode, String)>;
pub type EvalResult<'a> = Result<(Box<Environment<'a>>, Object), (ErrorCode, String)>;

#[derive(Debug, Clone)]
pub struct DynamicContext {
    pub item: Object,
    pub position: Option<usize>,
    pub last: Option<usize>,
}

impl DynamicContext {
    pub(crate) fn nothing() -> Self {
        Self {
            item: Object::Nothing,
            position: None,
            last: None,
        }
    }
}

pub(crate) fn eval_statements(statements: Vec<Statement>, env: Box<Environment>) -> Result<Object, (ErrorCode, String)> {

    let mut result = Object::Empty;

    let mut current_env = env;

    for statement in statements {
        let (new_env, new_result) = eval_statement(statement, current_env)?;
        current_env = new_env;

        result = new_result;

        if let &Object::Return(_) = &result {
            return Ok(result);
        }
    }

    Ok(result)
}

fn eval_statement(statement: Statement, env: Box<Environment>) -> EvalResult {
    match statement {
        Statement::Prolog(exprs) => eval_prolog(exprs, env),
        Statement::Program(expr) => expr.eval(env, &DynamicContext::nothing()),
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

#[allow(dead_code)]
enum Axis {
    ForwardChild,
    ForwardDescendant,
    ForwardAttribute,
    ForwardSelf,
    ForwardDescendantOrSelf,
    ForwardFollowingSibling,
    ForwardFollowing,

    ReverseParent,
    ReverseAncestor,
    ReversePrecedingSibling,
    ReversePreceding,
    ReverseAncestorOrSelf,
}

fn step_and_test<'a>(step: Axis, test: NameTest, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
    match &context.item {
        Object::Nothing => {
            panic!("XPDY0002")
        },
        Object::Node(node) => {
            match node {
                Node::Element { sequence, name, attributes, children } => {
                    match step {
                        Axis::ForwardChild => {
                            let mut result = vec![];
                            for child in children {
                                if test_node(&test, child) {
                                    result.push(Object::Node(child.clone()))
                                }
                            }

                            relax(env, result)
                        },
                        Axis::ForwardAttribute => {
                            let mut result = vec![];
                            for attribute in attributes {
                                if test_node(&test, attribute) {
                                    result.push(Object::Node(attribute.clone()))
                                }
                            }

                            relax(env, result)
                        }
                        _ => todo!()
                    }
                },
                _ => Ok((env, Object::Empty))
            }
        },
        _ => Ok((env, Object::Empty))
    }
}

fn test_node(test: &NameTest, node: &Node) -> bool {
    match test {
        NameTest { name: qname } => {
            match node {
                Node::Element { sequence, name, attributes, children } => {
                    qname.local_part == name.local_part && qname.url == name.url
                },
                Node::Attribute { sequence, name, value } => {
                    qname.local_part == name.local_part && qname.url == name.url
                },
                Node::Text { sequence, content } => false,
                _ => panic!("error {:?}", node)
            }
        },
        _ => panic!("error {:?}", test)
    }
}


fn eval_predicates<'a>(exprs: &Vec<Predicate>, env: Box<Environment<'a>>, value: Object, context: &DynamicContext) -> EvalResult<'a> {
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

pub(crate) fn object_to_integer(object: Object) -> Result<i128, (ErrorCode, String)> {
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
        Object::Node(node) => {
            let mut result = vec![];
            typed_value_of_node(node, &mut result);
            let num = result.join("");
            match num.parse() {
                Ok(v) => Ok(v),
                Err(..) => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", num)))
            }
        }
        _ => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", object)))
    }
}

// TODO: optimize!!!
pub(crate) fn object_to_iterator<'a>(object: &Object) -> Vec<Object> {
    match object {
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
        Object::Empty |
        Object::Node(..) |
        Object::Atomic(..) => {
            let mut result = Vec::with_capacity(1);
            result.push(object);
            result
        },
        Object::Range { min , max } => {
            let (it, count) = RangeIterator::create(min, max);
            let mut result = Vec::with_capacity(count.min(0) as usize);
            for item in it {
                result.push(item);
            }
            result
        },
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
            let env = Environment::new();

            let result = eval_statements(program, Box::new(env)).unwrap();

            assert_eq!(
                result,
                expected
            );
        } else {
            panic!("parse return error");
        }
    }
}
