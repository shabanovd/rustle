use crate::eval::{Environment, Object, Type, EvalResult, comparison, DynamicContext};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

// fn:deep-equal($parameter1 as item()*, $parameter2 as item()*) as xs:boolean
pub(crate) fn FN_DEEP_EQUAL_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_more(ItemType::Item),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_deep_equal
    )
}

// fn:deep-equal($parameter1 as item()*, $parameter2 as item()*, $collation as xs:string) as xs:boolean
pub(crate) fn FN_DEEP_EQUAL_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_deep_equal
    )
}

pub(crate) fn fn_deep_equal(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let o1 = arguments.remove(0);
    let o2 = arguments.remove(0);
    match comparison::deep_eq((&env, &o1), (&env, &o2)) {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(v)))),
        Err(e) => Err(e)
    }
}