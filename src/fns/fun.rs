use crate::eval::{Object, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

use crate::values::resolve_element_qname;
use crate::fns::call;
use crate::fns::strings::object_to_array;
use crate::parser::errors::ErrorCode;

// fn:function-lookup($name as xs:QName, $arity as xs:integer) as function(*)?
pub(crate) fn FN_FUNCTION_LOOKUP() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_QNAME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Function { args: None, st: None })
        ),
        fn_function_lookup
    )
}

pub(crate) fn fn_function_lookup(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:function-name($func as function(*)) as xs:QName?
pub(crate) fn FN_FUNCTION_NAME() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Function { args: None, st: None })].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        fn_function_name
    )
}

pub(crate) fn fn_function_name(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:function-arity($func as function(*)) as xs:integer
pub(crate) fn FN_FUNCTION_ARITY() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Function { args: None, st: None })].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_function_arity
    )
}

pub(crate) fn fn_function_arity(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// 16.2 Basic higher-order functions

// fn:for-each($seq as item()*, $action as function(item()) as item()*) as item()*
pub(crate) fn FN_FOR_EACH() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_for_each
    )
}

pub(crate) fn fn_for_each(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Function { parameters, st, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

// fn:filter($seq as item()*, $f as function(item()) as xs:boolean) as item()*
pub(crate) fn FN_FILTER() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_for_each
    )
}

pub(crate) fn fn_filter(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Function { parameters, st, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

// fn:fold-left($seq as item()*, $zero as item()*, $f as function(item()*, item()) as item()*) as item()*
pub(crate) fn FN_FOLD_LEFT() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::zero_or_more(ItemType::Item),
                        SequenceType::exactly_one(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_fold_left
    )
}

pub(crate) fn fn_fold_left(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [seq, Object::Array(array), Object::FunctionRef { name, arity }] => {
            let mut result = array.clone();

            let mut current_env = env;

            for item in seq.clone().into_iter() {
                let arguments = vec![Object::Array(result), item];
                if *arity != arguments.len() {
                    todo!("raise error")
                }
                let (new_env, obj) = call(current_env, name.clone(), arguments, context)?;
                current_env = new_env;

                result = object_to_array(obj);
            }

            if result.is_empty() {
                Ok((current_env, Object::Empty))
            } else {
                Ok((current_env, Object::Array(result)))
            }
        },
        _ => panic!("error")
    }
}

// fn:fold-right($seq as item()*, $zero as item()*, $f as function(item(), item()*) as item()*) as item()*
pub(crate) fn FN_FOLD_RIGHT() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::Item),
                        SequenceType::zero_or_more(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_fold_right
    )
}

pub(crate) fn fn_fold_right(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Function { parameters, st, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

// fn:for-each-pair($seq1 as item()*, $seq2 as item()*, $action as function(item(), item()) as item()*) as item()*
pub(crate) fn FN_FOR_EACH_PAIR() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::Item),
                        SequenceType::zero_or_more(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_for_each_pair
    )
}

pub(crate) fn fn_for_each_pair(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Function { parameters, st, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

// fn:sort($input as item()*) as item()*
pub(crate) fn FN_SORT_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_sort
    )
}

// fn:sort($input as item()*, $collation as xs:string?) as item()*
pub(crate) fn FN_SORT_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_sort
    )
}

// fn:sort($input as item()*, $collation as xs:string?, $key as function(item()) as xs:anyAtomicType*) as item()*
pub(crate) fn FN_SORT_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::AnyAtomicType)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_sort
    )
}

pub(crate) fn fn_sort(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Function { parameters, st, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

// fn:apply($function as function(*), $array as array(*)) as item()*
pub(crate) fn FN_APPLY() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Function { args: None, st: None }),
                SequenceType::exactly_one(ItemType::Array(None)),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_apply
    )
}

pub(crate) fn fn_apply(env: Box<Environment>, mut arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let mut current_env = env;

    let arg1 = arguments.remove(0);
    let arg2 = arguments.remove(0);

    match (arg1, arg2) {
        (Object::Function { parameters, st, body }, Object::Array( arguments )) => {

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut fn_env = current_env.next();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {

                let name = resolve_element_qname(&parameter.name, &fn_env);

                fn_env.set_variable(name, argument);
            }

            let (new_env, result) = body.eval(fn_env, &DynamicContext::nothing())?;
            current_env = new_env.prev();

            Ok((current_env, result))
        },
        (Object::FunctionRef { name, arity }, Object::Array( arguments )) => {
            let fun = current_env.functions.get(&name, arity);

            return if let Some(((params, st), fun)) = fun {
                fun(current_env, arguments, context)
            } else {
                panic!("no function {:?}#{:?}", name, arity)
            }
        },
        _ => panic!("error")
    }
}

// fn:error() as none
pub(crate) fn FN_ERROR_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::none()
        ),
        fn_error
    )
}

// fn:error($code as xs:QName?) as none
pub(crate) fn FN_ERROR_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))].to_vec(),
            SequenceType::none()
        ),
        fn_error
    )
}

// fn:error($code as xs:QName?, $description as xs:string) as none
pub(crate) fn FN_ERROR_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::none()
        ),
        fn_error
    )
}

// fn:error($code as xs:QName?, $description as xs:string, $error-object as item()*) as none
pub(crate) fn FN_ERROR_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_more(ItemType::Item),
            ].to_vec(),
            SequenceType::none()
        ),
        fn_error
    )
}

pub(crate) fn fn_error(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [] => {
            Err((ErrorCode::FOER0000, String::new()))
        },
        _ => todo!("arguments: {:?}", arguments)
    }
}
