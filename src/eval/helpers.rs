use crate::eval::{Environment, Object, EvalResult, object_owned_to_sequence};

pub(crate) fn relax(env: Box<Environment>, items: Vec<Object>) -> EvalResult {
    if items.len() == 0 {
        Ok((env, Object::Empty))
    } else if items.len() == 1 {
        if let Some(item) = items.into_iter().next() {
            Ok((env, item))
        } else {
            panic!("internal error")
        }
    } else {
        Ok((env, Object::Sequence(items)))
    }
}

pub(crate) fn process_items<F>(env: Box<Environment>, object: Object, op: F) -> EvalResult
    where F: Fn(Box<Environment>, Object) -> EvalResult
{
    let mut current_env = env;
    let mut result = vec![];

    let items = object_owned_to_sequence(object);
    for item in items {
        let (new_env, object) = op(current_env, item)?;
        current_env = new_env;

        result.push(object);
    }

    relax(current_env, result)
}

pub(crate) fn join_sequences(result: &mut Vec<Object>, seq: Vec<Object>) {
    // space allocation
    result.reserve(seq.len());

    for item in seq {
        match item {
            Object::Empty => {},
            Object::Sequence(items) => {
                join_sequences(result, items)
            },
            Object::Node(..) => result.push(item),
            _ => panic!("XPTY0004: item is not a node")
        }
    }
}

pub(crate) fn relax_sequences(result: &mut Vec<Object>, seq: Vec<Object>) {
    // space allocation
    result.reserve(seq.len());

    for item in seq {
        match item {
            Object::Nothing |
            Object::Empty => {},
            Object::Sequence(items) => {
                relax_sequences(result, items)
            },
            _ => result.push(item)
        }
    }
}

pub fn sort_and_dedup(seq: &mut Vec<Object>) {
    seq.sort();
    seq.dedup();
}