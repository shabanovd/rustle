use std::convert::TryFrom;
use std::iter::FromIterator;
use crate::eval::{Environment, Object, Type, DynamicContext, EvalResult};
use crate::eval::helpers::relax;
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;
use crate::namespaces::*;

use crate::serialization::object_to_string;
use crate::serialization::to_string::_object_to_string;
use crate::parser::errors::ErrorCode;
use crate::values::Types;

// fn:string() as xs:string
pub(crate) fn FN_STRING_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_string
    )
}

// fn:string($arg as item()?) as xs:string
pub(crate) fn FN_STRING_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_string
    )
}

pub(crate) fn fn_string(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str))))
}

// fn:codepoints-to-string($arg as xs:integer*) as xs:string
pub(crate) fn FN_CODEPOINTS_TO_STRING() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_INTEGER.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_codepoints_to_string
    )
}

pub(crate) fn fn_codepoints_to_string(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let codes = arguments.remove(0);
    match codes {
        Object::Sequence(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Object::Atomic(Type::Integer(num)) => {
                        if let Ok(code) = u32::try_from(num) {
                            if let Ok(ch) = char::try_from(code) {
                                result.push(ch)
                            } else {
                                todo!("raise error?")
                            }
                        } else {
                            todo!("raise error?")
                        }
                    }
                    _ => todo!()
                }
            }
            println!("{:?}", result);
            let str = String::from_iter(result);
            println!("{:?}", str.chars());
            Ok((env, Object::Atomic(Type::String(str))))
        }
        _ => todo!()
    }
}

// fn:string-to-codepoints($arg as xs:string?) as xs:integer*
pub(crate) fn FN_STRING_TO_CODEPOINTS() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_string_to_codepoints
    )
}

pub(crate) fn fn_string_to_codepoints(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let processing = |env: Box<Environment>, str: String| {
        let mut codes = Vec::with_capacity(str.len());
        for char in str.chars() {
            // let code = char as u32;
            codes.push(Object::Atomic(Type::Integer(char as i128)));
        }
        relax(env, codes)
    };

    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(t)] => {
            if let Type::String(str) = t.convert(Types::String)? {
                processing(env, str)
            } else {
                todo!("raise error?")
            }
        }
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => processing(env, str),
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!("{:?}", arguments)
    }
}

// fn:concat($arg1 as xs:anyAtomicType?, $arg2 as xs:anyAtomicType?, ...) as xs:string
pub(crate) fn FN_CONCAT(arity: usize) -> FUNCTION {
    let params = vec![SequenceType::zero_or_one(ItemType::AnyAtomicType); arity];
    (
        (
            params,
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_concat
    )
}

pub(crate) fn fn_concat(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let str = arguments.iter()
        .map(|item| object_to_string(&env, item))
        .collect();

    Ok((env, Object::Atomic(Type::String(str))))
}

// fn:string-join($arg1 as xs:anyAtomicType*) as xs:string
pub(crate) fn FN_STRING_JOIN_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_string_join
    )
}

// fn:string-join($arg1 as xs:anyAtomicType*, $arg2 as xs:string) as xs:string
pub(crate) fn FN_STRING_JOIN_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_string_join
    )
}

pub(crate) fn fn_string_join(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let str = if let Some(item) = arguments.get(0) {
        if let Some(sep) = arguments.get(1) {
            let sep = object_to_string(&env, sep);
            _object_to_string(&env, item, true, sep.as_str())
        } else {
            _object_to_string(&env, item, true, " ")
        }
    } else {
        return Err((ErrorCode::TODO, format!("got {:?} arguments, but expected 1 or 2", arguments.len())));
    };

    Ok((env, Object::Atomic(Type::String(str))))
}

// fn:substring($sourceString as xs:string?, $start as xs:double) as xs:string
pub(crate) fn FN_SUBSTRING_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring
    )
}

// fn:substring($sourceString as xs:string?, $start as xs:double, $length as xs:double) as xs:string
pub(crate) fn FN_SUBSTRING_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring
    )
}

pub(crate) fn fn_substring(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:string-length() as xs:integer
pub(crate) fn FN_STRING_LENGTH_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_string_length
    )
}

// fn:string-length($arg as xs:string?) as xs:integer
pub(crate) fn FN_STRING_LENGTH_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_string_length
    )
}

pub(crate) fn fn_string_length(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::Integer(str.len() as i128))))
}

// fn:normalize-space() as xs:string
pub(crate) fn FN_NORMALIZE_SPACE_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_normalize_space
    )
}

// fn:normalize-space($arg as xs:string?) as xs:string
pub(crate) fn FN_NORMALIZE_SPACE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_normalize_space
    )
}

pub(crate) fn fn_normalize_space(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let mut str = object_to_string(&env, item);
    str = str.trim().to_string();

    // TODO replacing sequences of one or more adjacent whitespace characters with a single space

    Ok((env, Object::Atomic(Type::String(str))))
}

// fn:normalize-unicode($arg as xs:string?) as xs:string
pub(crate) fn FN_NORMALIZE_UNICODE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_normalize_unicode
    )
}

// fn:normalize-unicode($arg as xs:string?, $normalizationForm as xs:string) as xs:string
pub(crate) fn FN_NORMALIZE_UNICODE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_normalize_unicode
    )
}

pub(crate) fn fn_normalize_unicode(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:upper-case($arg as xs:string?) as xs:string
pub(crate) fn FN_UPPER_CASE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_upper_case
    )
}

pub(crate) fn fn_upper_case(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str.to_uppercase()))))
}

// fn:lower-case($arg as xs:string?) as xs:string
pub(crate) fn FN_LOWER_CASE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_lower_case
    )
}

pub(crate) fn fn_lower_case(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str.to_lowercase()))))
}

// fn:translate($arg as xs:string?, $mapString as xs:string, $transString as xs:string) as xs:string
pub(crate) fn FN_TRANSLATE() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_translate
    )
}

pub(crate) fn fn_translate(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:contains($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
pub(crate) fn FN_CONTAINS_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_contains
    )
}

// fn:contains($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean
pub(crate) fn FN_CONTAINS_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_contains
    )
}

pub(crate) fn fn_contains(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let string = arguments.get(0).unwrap();
    let pattern = arguments.get(1).unwrap();

    // TODO handle $collation

    let string = object_to_string(&env, string);
    let pattern = object_to_string(&env, pattern);

    let result = string.contains(&pattern);

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

// fn:starts-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
pub(crate) fn FN_STARTS_WITH_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_starts_with
    )
}

// fn:starts-with($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean
pub(crate) fn FN_STARTS_WITH_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_starts_with
    )
}

pub(crate) fn fn_starts_with(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {

    let string = arguments.get(0).unwrap();
    let pattern = arguments.get(1).unwrap();

    let string = object_to_string(&env, string);
    let pattern = object_to_string(&env, pattern);

    let result = string.starts_with(&pattern);

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

// fn:ends-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
pub(crate) fn FN_ENDS_WITH_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_ends_with
    )
}

// fn:ends-with($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean
pub(crate) fn FN_ENDS_WITH_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_ends_with
    )
}

pub(crate) fn fn_ends_with(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {

    let string = arguments.get(0).unwrap();
    let pattern = arguments.get(1).unwrap();

    let string = object_to_string(&env, string);
    let pattern = object_to_string(&env, pattern);

    let result = string.ends_with(&pattern);

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

// fn:substring-before($arg1 as xs:string?, $arg2 as xs:string?) as xs:string
pub(crate) fn FN_SUBSTRING_BEFORE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring_before
    )
}

// fn:substring-before($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:string
pub(crate) fn FN_SUBSTRING_BEFORE_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring_before
    )
}

pub(crate) fn fn_substring_before(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:substring-after($arg1 as xs:string?, $arg2 as xs:string?) as xs:string
pub(crate) fn FN_SUBSTRING_AFTER_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring_after
    )
}

// fn:substring-after($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:string
pub(crate) fn FN_SUBSTRING_AFTER_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_substring_after
    )
}

pub(crate) fn fn_substring_after(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:matches($input as xs:string?, $pattern as xs:string) as xs:boolean
pub(crate) fn FN_MATCHES_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_matches
    )
}

// fn:matches($input as xs:string?, $pattern as xs:string, $flags as xs:string) as xs:boolean
pub(crate) fn FN_MATCHES_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_matches
    )
}

pub(crate) fn fn_matches(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:replace($input as xs:string?, $pattern as xs:string, $replacement as xs:string) as xs:string
pub(crate) fn FN_REPLACE_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_replace
    )
}

// fn:replace($input as xs:string?, $pattern as xs:string, $replacement as xs:string, $flags as xs:string) as xs:string
pub(crate) fn FN_REPLACE_4() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_replace
    )
}

pub(crate) fn fn_replace(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:tokenize($input as xs:string?) as xs:string*
pub(crate) fn FN_TOKENIZE_1() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_tokenize
    )
}

// fn:tokenize($input as xs:string?, $pattern as xs:string) as xs:string*
pub(crate) fn FN_TOKENIZE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_tokenize
    )
}

// fn:tokenize($input as xs:string?, $pattern as xs:string, $flags as xs:string) as xs:string*
pub(crate) fn FN_TOKENIZE_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_tokenize
    )
}

pub(crate) fn fn_tokenize(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:analyze-string($input as xs:string?, $pattern as xs:string) as element(fn:analyze-string-result)
pub(crate) fn FN_ANALYZE_STRING_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::element_ns(&XPATH_FUNCTIONS, "analyze-string-result"))
        ),
        fn_analyze_string
    )
}

// fn:analyze-string($input as xs:string?, $pattern as xs:string, $flags as xs:string) as element(fn:analyze-string-result)
pub(crate) fn FN_ANALYZE_STRING_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::element_ns(&XPATH_FUNCTIONS, "analyze-string-result"))
        ),
        fn_analyze_string
    )
}

pub(crate) fn fn_analyze_string(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn object_to_array(object: Object) -> Vec<Object> {
    match object {
        Object::Array(array) => array,
        _ => panic!("TODO object_to_array {:?}", object)
    }
}
