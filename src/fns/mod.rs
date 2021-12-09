use std::collections::HashMap;
use crate::eval::{Object, DynamicContext, EvalResult, ErrorInfo, Type};
use crate::eval::Environment;
use crate::namespaces::*;
use crate::values::{QName, QNameResolved, resolve_element_qname, Types};

mod fun;
mod sequences;
mod qname;
mod boolean;
mod strings;
mod types;
mod datetime;
mod nodes;
mod context;
mod comparison;
mod math;
mod map;
mod array;
mod aggregates;

pub use crate::fns::boolean::object_to_bool;

use crate::parser::errors::ErrorCode;
use crate::eval::expression::Expression;
use crate::eval::sequence_type::{ItemType, SequenceType, XS_NOTATION, XS_QNAME};
use crate::fns::types::*;

pub type FUNCTION = ((Vec<SequenceType>, SequenceType), fn(Box<Environment>, Vec<Object>, &DynamicContext) -> EvalResult);

#[derive(Clone)]
pub struct Function {
    name: QNameResolved,
    parameters: Vec<Param>,
    st: Option<SequenceType>,
    body: Box<dyn Expression>,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: QName,
    pub sequence_type: Option<SequenceType> // TODO: new type?
}

pub enum Occurrence {
    Arity(usize),
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}

#[derive(Clone)]
pub struct FunctionsRegister {
    functions: HashMap<QNameResolved, HashMap<usize, FUNCTION>>,
    declared: HashMap<QNameResolved, HashMap<usize, Function>>,
}

impl FunctionsRegister {
    pub(crate) fn new() -> Self {
        let mut instance = FunctionsRegister {
            functions: HashMap::new(),
            declared: HashMap::new(),
        };

        instance.register(&*SCHEMA.uri, "untypedAtomic", 1, types::FN_XS_UNTYPED_ATOMIC());

        instance.register(&*SCHEMA.uri, "string", 1, types::FN_XS_STRING());
        instance.register(&*SCHEMA.uri, "normalizedString", 1, types::FN_XS_NORMALIZED_STRING());

        instance.register(&*SCHEMA.uri, "boolean", 1, types::FN_XS_BOOLEAN());

        instance.register(&*SCHEMA.uri, "unsignedByte", 1, types::FN_XS_UNSIGNED_BYTE());
        instance.register(&*SCHEMA.uri, "unsignedShort", 1, types::FN_XS_UNSIGNED_SHORT());
        instance.register(&*SCHEMA.uri, "unsignedInt", 1, types::FN_XS_UNSIGNED_INT());
        instance.register(&*SCHEMA.uri, "unsignedLong", 1, types::FN_XS_UNSIGNED_LONG());

        instance.register(&*SCHEMA.uri, "byte", 1, types::FN_XS_BYTE());
        instance.register(&*SCHEMA.uri, "short", 1, types::FN_XS_SHORT());
        instance.register(&*SCHEMA.uri, "int", 1, types::FN_XS_INT());
        instance.register(&*SCHEMA.uri, "long", 1, types::FN_XS_LONG());

        instance.register(&*SCHEMA.uri, "positiveInteger", 1, types::FN_XS_POSITIVE_INTEGER());
        instance.register(&*SCHEMA.uri, "nonNegativeInteger", 1, types::FN_XS_NON_NEGATIVE_INTEGER());
        instance.register(&*SCHEMA.uri, "nonPositiveInteger", 1, types::FN_XS_NON_POSITIVE_INTEGER());
        instance.register(&*SCHEMA.uri, "negativeInteger", 1, types::FN_XS_NEGATIVE_INTEGER());

        instance.register(&*SCHEMA.uri, "integer", 1, types::FN_XS_INTEGER());
        instance.register(&*SCHEMA.uri, "decimal", 1, types::FN_XS_DECIMAL());
        instance.register(&*SCHEMA.uri, "float", 1, types::FN_XS_FLOAT());
        instance.register(&*SCHEMA.uri, "double", 1, types::FN_XS_DOUBLE());

        instance.register(&*SCHEMA.uri, "ID", 1, types::FN_XS_ID());
        instance.register(&*SCHEMA.uri, "IDREF", 1, types::FN_XS_IDREF());
        instance.register(&*SCHEMA.uri, "ENTITY", 1, types::FN_XS_ENTITY());

        instance.register(&*SCHEMA.uri, "NCName", 1, types::FN_XS_NCNAME());

        instance.register(&*SCHEMA.uri, "anyURI", 1, types::FN_XS_ANY_URI());

        instance.register(&*SCHEMA.uri, "time", 1, types::FN_XS_TIME());
        instance.register(&*SCHEMA.uri, "date", 1, types::FN_XS_DATE());
        instance.register(&*SCHEMA.uri, "dateTime", 1, types::FN_XS_DATE_TIME());
        instance.register(&*SCHEMA.uri, "yearMonthDuration", 1, types::FN_XS_YEAR_MONTH_DURATION());
        instance.register(&*SCHEMA.uri, "dayTimeDuration", 1, types::FN_XS_DAY_TIME_DURATION());
        instance.register(&*SCHEMA.uri, "duration", 1, types::FN_XS_DURATION());

        instance.register(&*SCHEMA.uri, "gYearMonth", 1, types::FN_XS_G_YEAR_MONTH());
        instance.register(&*SCHEMA.uri, "gYear", 1, types::FN_XS_G_YEAR());
        instance.register(&*SCHEMA.uri, "gMonthDay", 1, types::FN_XS_G_MONTH_DAY());
        instance.register(&*SCHEMA.uri, "gDay", 1, types::FN_XS_G_DAY());
        instance.register(&*SCHEMA.uri, "gMonth", 1, types::FN_XS_G_MONTH());

        instance.register(&*SCHEMA.uri, "hexBinary", 1, types::FN_XS_HEX_BINARY());
        instance.register(&*SCHEMA.uri, "base64Binary", 1, types::FN_XS_BASE64_BINARY());

        instance.register(&*SCHEMA.uri, "QName", 1, types::FN_XS_QNAME());
        instance.register(&*SCHEMA.uri, "token", 1, types::FN_XS_TOKEN());
        instance.register(&*SCHEMA.uri, "language", 1, types::FN_XS_LANGUAGE());
        instance.register(&*SCHEMA.uri, "NMTOKEN", 1, types::FN_XS_NMTOKEN());
        instance.register(&*SCHEMA.uri, "Name", 1, types::FN_XS_NAME());

        instance.register(&*XPATH_FUNCTIONS.uri, "resolve-QName", 2, qname::FN_RESOLVE_QNAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "QName", 2, qname::FN_QNAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "prefix-from-QName", 1, qname::FN_PREFIX_FROM_QNAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name-from-QName", 1, qname::FN_LOCAL_NAME_FROM_QNAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri-from-QName", 1, qname::FN_NAMESPACE_URI_FROM_QNAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri-for-prefix", 1, qname::FN_NAMESPACE_URI_FOR_PREFIX());
        instance.register(&*XPATH_FUNCTIONS.uri, "in-scope-prefixes", 1, qname::FN_IN_SCOPE_PREFIXES());

        instance.register(&*XPATH_FUNCTIONS.uri, "node-name", 0, qname::FN_NODE_NAME_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "node-name", 1, qname::FN_NODE_NAME_1());

//        instance.register("op", "same-key", 2, map::map_merge);
        instance.register(&*XPATH_MAP.uri, "merge", 1, map::FN_MAP_MERGE_1());
        instance.register(&*XPATH_MAP.uri, "merge", 2, map::FN_MAP_MERGE_2());
        instance.register(&*XPATH_MAP.uri, "size", 1, map::FN_MAP_SIZE());
        instance.register(&*XPATH_MAP.uri, "contains", 2, map::FN_MAP_CONTAINS());
        instance.register(&*XPATH_MAP.uri, "get", 2, map::FN_MAP_GET());
        instance.register(&*XPATH_MAP.uri, "find", 2, map::FN_MAP_FIND());
        instance.register(&*XPATH_MAP.uri, "put", 3, map::FN_MAP_PUT());
        instance.register(&*XPATH_MAP.uri, "find", 2, map::FN_MAP_FIND());
        instance.register(&*XPATH_MAP.uri, "entry", 2, map::FN_MAP_ENTRY());
        instance.register(&*XPATH_MAP.uri, "remove", 2, map::FN_MAP_REMOVE());
        instance.register(&*XPATH_MAP.uri, "for-each", 2, map::FN_MAP_FOR_EACH());

        instance.register(&*XPATH_ARRAY.uri, "size", 1, array::FN_ARRAY_SIZE());
        instance.register(&*XPATH_ARRAY.uri, "get", 2, array::FN_ARRAY_GET());
        instance.register(&*XPATH_ARRAY.uri, "put", 3, array::FN_ARRAY_PUT());
        instance.register(&*XPATH_ARRAY.uri, "append", 2, array::FN_ARRAY_APPEND());
        instance.register(&*XPATH_ARRAY.uri, "subarray", 2, array::FN_ARRAY_SUBARRAY_2());
        instance.register(&*XPATH_ARRAY.uri, "subarray", 3, array::FN_ARRAY_SUBARRAY_2());
        instance.register(&*XPATH_ARRAY.uri, "insert-before", 3, array::FN_ARRAY_INSERT_BEFORE());
        instance.register(&*XPATH_ARRAY.uri, "head", 1, array::FN_ARRAY_HEAD());
        instance.register(&*XPATH_ARRAY.uri, "tail", 1, array::FN_ARRAY_TAIL());
        instance.register(&*XPATH_ARRAY.uri, "reverse", 1, array::FN_ARRAY_REVERSE());
        instance.register(&*XPATH_ARRAY.uri, "join", usize::MAX, array::FN_ARRAY_JOIN());
        instance.register(&*XPATH_ARRAY.uri, "for-each", 2, array::FN_ARRAY_FOR_EACH());
        instance.register(&*XPATH_ARRAY.uri, "filter", 2, array::FN_ARRAY_FILTER());
        instance.register(&*XPATH_ARRAY.uri, "fold-left", 3, array::FN_ARRAY_FOLD_LEFT());
        instance.register(&*XPATH_ARRAY.uri, "fold-right", 3, array::FN_ARRAY_FOLD_RIGHT());
        instance.register(&*XPATH_ARRAY.uri, "for-each-pair", 3, array::FN_ARRAY_FOR_EACH_PAIR());
        instance.register(&*XPATH_ARRAY.uri, "sort", 1, array::FN_ARRAY_SORT_1());
        instance.register(&*XPATH_ARRAY.uri, "sort", 2, array::FN_ARRAY_SORT_2());
        instance.register(&*XPATH_ARRAY.uri, "sort", 3, array::FN_ARRAY_SORT_3());
        instance.register(&*XPATH_ARRAY.uri, "flatten", 1, array::FN_ARRAY_FLATTEN());

        instance.register(&*XPATH_FUNCTIONS.uri, "current-dateTime", 0, datetime::FN_CURRENT_DATE_TIME());
        instance.register(&*XPATH_FUNCTIONS.uri, "current-date", 0, datetime::FN_CURRENT_DATE());
        instance.register(&*XPATH_FUNCTIONS.uri, "current-time", 0, datetime::FN_CURRENT_TIME());
        instance.register(&*XPATH_FUNCTIONS.uri, "year-from-date", 1, datetime::FN_YEAR_FROM_DATE());
        instance.register(&*XPATH_FUNCTIONS.uri, "month-from-date", 1, datetime::FN_MONTH_FROM_DATE());
        instance.register(&*XPATH_FUNCTIONS.uri, "day-from-date", 1, datetime::FN_DAY_FROM_DATE());
        instance.register(&*XPATH_FUNCTIONS.uri, "days-from-duration", 1, datetime::FN_DAYS_FROM_DURATION());

        instance.register(&*XPATH_FUNCTIONS.uri, "timezone-from-time", 1, datetime::FN_TIMEZONE_FROM_DATE_TIME());
        instance.register(&*XPATH_FUNCTIONS.uri, "timezone-from-time", 1, datetime::FN_TIMEZONE_FROM_DATE());
        instance.register(&*XPATH_FUNCTIONS.uri, "timezone-from-time", 1, datetime::FN_TIMEZONE_FROM_TIME());

        instance.register(&*XPATH_FUNCTIONS.uri, "for-each", 2, fun::FN_FOR_EACH());
        instance.register(&*XPATH_FUNCTIONS.uri, "filter", 2, fun::FN_FILTER());
        instance.register(&*XPATH_FUNCTIONS.uri, "fold-left", 3, fun::FN_FOLD_LEFT());
        instance.register(&*XPATH_FUNCTIONS.uri, "fold-right", 3, fun::FN_FOLD_RIGHT());
        instance.register(&*XPATH_FUNCTIONS.uri, "for-each-pair", 3, fun::FN_FOR_EACH_PAIR());
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 1, fun::FN_SORT_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 2, fun::FN_SORT_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 3, fun::FN_SORT_3());
        instance.register(&*XPATH_FUNCTIONS.uri, "apply", 2, fun::FN_APPLY());

        instance.register(&*XPATH_FUNCTIONS.uri, "error", 0, fun::FN_ERROR_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 1, fun::FN_ERROR_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 2, fun::FN_ERROR_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 3, fun::FN_ERROR_3());

        instance.register(&*XPATH_FUNCTIONS.uri, "format-number", 2, math::FN_MATH_FORMAT_NUMBER_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "format-number", 3, math::FN_MATH_FORMAT_NUMBER_3());

        instance.register(&*XPATH_FUNCTIONS.uri, "number", 0, math::FN_MATH_NUMBER_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "number", 1, math::FN_MATH_NUMBER_1());

        instance.register(&*XPATH_FUNCTIONS.uri, "count", 1, aggregates::FN_COUNT());
        instance.register(&*XPATH_FUNCTIONS.uri, "avg", 1, aggregates::FN_AVG());
        instance.register(&*XPATH_FUNCTIONS.uri, "max", 1, aggregates::FN_MAX_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "max", 2, aggregates::FN_MAX_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "min", 1, aggregates::FN_MIN_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "min", 2, aggregates::FN_MIN_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "sum", 1, aggregates::FN_SUM_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "sum", 2, aggregates::FN_SUM_2());

        instance.register(&*XPATH_MATH.uri, "pi", 0, math::FN_MATH_PI());

        instance.register(&*XPATH_FUNCTIONS.uri, "abs", 1, math::FN_MATH_ABS());
        instance.register(&*XPATH_FUNCTIONS.uri, "ceiling", 1, math::FN_MATH_CEILING());
        instance.register(&*XPATH_FUNCTIONS.uri, "floor", 1, math::FN_MATH_FLOOR());
        instance.register(&*XPATH_FUNCTIONS.uri, "round", 1, math::FN_MATH_ROUND_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "round", 2, math::FN_MATH_ROUND_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "round-half-to-even", 1, math::FN_MATH_ROUND_HALF_TO_EVEN_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "round-half-to-even", 2, math::FN_MATH_ROUND_HALF_TO_EVEN_2());

        instance.register(&*XPATH_FUNCTIONS.uri, "boolean", 1, boolean::FN_BOOLEAN());
        instance.register(&*XPATH_FUNCTIONS.uri, "true", 0, boolean::FN_TRUE());
        instance.register(&*XPATH_FUNCTIONS.uri, "false", 0, boolean::FN_FALSE());
        instance.register(&*XPATH_FUNCTIONS.uri, "not", 1, boolean::FN_NOT());

        instance.register(&*XPATH_FUNCTIONS.uri, "string", 0, strings::FN_STRING_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "string", 1, strings::FN_STRING_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "codepoints-to-string", 1, strings::FN_CODEPOINTS_TO_STRING());
        instance.register(&*XPATH_FUNCTIONS.uri, "string-to-codepoints", 1, strings::FN_STRING_TO_CODEPOINTS());
        instance.register(&*XPATH_FUNCTIONS.uri, "string-join", 1, strings::FN_STRING_JOIN_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "string-join", 2, strings::FN_STRING_JOIN_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "string-length", 0, strings::FN_STRING_LENGTH_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "string-length", 1, strings::FN_STRING_LENGTH_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "normalize-space", 0, strings::FN_NORMALIZE_SPACE_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "normalize-space", 1, strings::FN_NORMALIZE_SPACE_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "upper-case", 1, strings::FN_UPPER_CASE());
        instance.register(&*XPATH_FUNCTIONS.uri, "lower-case", 1, strings::FN_LOWER_CASE());
        instance.register(&*XPATH_FUNCTIONS.uri, "starts-with", 2, strings::FN_STARTS_WITH_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "starts-with", 3, strings::FN_STARTS_WITH_3());
        instance.register(&*XPATH_FUNCTIONS.uri, "ends-with", 2, strings::FN_ENDS_WITH_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "ends-with", 3, strings::FN_ENDS_WITH_3());

        instance.register(&*XPATH_FUNCTIONS.uri, "position", 0, sequences::FN_POSITION());
        instance.register(&*XPATH_FUNCTIONS.uri, "last", 0, sequences::FN_LAST());
        instance.register(&*XPATH_FUNCTIONS.uri, "default-collation", 0, context::FN_DEFAULT_COLLATION());
        instance.register(&*XPATH_FUNCTIONS.uri, "default-language", 0, context::FN_DEFAULT_LANGUAGE());
        instance.register(&*XPATH_FUNCTIONS.uri, "static-base-uri", 0, context::FN_STATIC_BASE_URI());

        instance.register(&*XPATH_FUNCTIONS.uri, "function-lookup", 0, fun::FN_FUNCTION_LOOKUP());
        instance.register(&*XPATH_FUNCTIONS.uri, "function-name", 0, fun::FN_FUNCTION_NAME());
        instance.register(&*XPATH_FUNCTIONS.uri, "function-arity", 0, fun::FN_FUNCTION_ARITY());

        instance.register(&*XPATH_FUNCTIONS.uri, "data", 0, sequences::FN_DATA_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "data", 1, sequences::FN_DATA_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "empty", 1, sequences::FN_EMPTY());
        instance.register(&*XPATH_FUNCTIONS.uri, "exists", 1, sequences::FN_EXISTS());
        instance.register(&*XPATH_FUNCTIONS.uri, "remove", 2, sequences::FN_REMOVE());
        instance.register(&*XPATH_FUNCTIONS.uri, "reverse", 1, sequences::FN_REVERSE());
        instance.register(&*XPATH_FUNCTIONS.uri, "subsequence", 2, sequences::FN_SUBSEQUENCE_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "subsequence", 3, sequences::FN_SUBSEQUENCE_3());

        instance.register(&*XPATH_FUNCTIONS.uri, "zero-or-one", 1, sequences::FN_ZERO_OR_ONE());
        instance.register(&*XPATH_FUNCTIONS.uri, "one-or-more", 1, sequences::FN_ONE_OR_MORE());
        instance.register(&*XPATH_FUNCTIONS.uri, "exactly-one", 1, sequences::FN_EXACTLY_ONE());

        instance.register(&*XPATH_FUNCTIONS.uri, "deep-equal", 2, comparison::FN_DEEP_EQUAL_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "deep-equal", 3, comparison::FN_DEEP_EQUAL_3());

        instance.register(&*XPATH_FUNCTIONS.uri, "name", 0, nodes::FN_NAME_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "name", 1, nodes::FN_NAME_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name", 0, nodes::FN_LOCAL_NAME_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name", 1, nodes::FN_LOCAL_NAME_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri", 0, nodes::FN_NAMESPACE_URI_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri", 1, nodes::FN_NAMESPACE_URI_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "lang", 1, nodes::FN_LANG_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "lang", 2, nodes::FN_LANG_2());
        instance.register(&*XPATH_FUNCTIONS.uri, "root", 0, nodes::FN_ROOT_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "root", 1, nodes::FN_ROOT_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "path", 0, nodes::FN_PATH_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "path", 1, nodes::FN_PATH_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "has-children", 0, nodes::FN_HAS_CHILDREN_0());
        instance.register(&*XPATH_FUNCTIONS.uri, "has-children", 1, nodes::FN_HAS_CHILDREN_1());
        instance.register(&*XPATH_FUNCTIONS.uri, "innermost", 1, nodes::FN_INNERMOST());
        instance.register(&*XPATH_FUNCTIONS.uri, "outermost", 1, nodes::FN_OUTERMOST());

        instance
    }

    pub(crate) fn register<S: Into<String>>(&mut self, uri: S, local_part: &str, arity: usize, fun: FUNCTION) {
        self.functions.entry(QNameResolved { url: uri.into(), local_part: local_part.into() })
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
    }

    pub(crate) fn put(&mut self, name: QNameResolved, parameters: Vec<Param>, st: Option<SequenceType>, body: Box<dyn Expression>) {
        self.declared.entry(name.clone())
            .or_insert_with(HashMap::new)
            .insert(parameters.len(), Function { name, parameters, st, body });
    }

    pub(crate) fn get(&self, qname: &QNameResolved, arity: usize) -> Option<FUNCTION> {
        if let Some(list) = self.functions.get(qname) {
            if let Some(rf) = list.get(&arity) {
                Some(rf.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn declared(&self, qname: &QNameResolved, arity: usize) -> Option<&Function> {
        // println!("function get {:?} {:?} {:?}", uri, local_part, arity);
        if let Some(list) = self.declared.get(qname) {
            // println!("function list {:?}", list.len());
            //TODO: fix it!
            let rf = list.get(&arity).unwrap();
            Some(rf)
        } else {
            // println!("function list NONE");
            None
        }
    }
}

pub(crate) fn cascade(env: &Environment, arguments: Vec<Object>, params: Vec<SequenceType>) -> Result<Vec<Object>, ErrorInfo> {
    if arguments.len() != params.len() {
        todo!("raise error?")
    } else {
        let mut args = Vec::with_capacity(arguments.len());
        for (arg, st) in arguments.into_iter()
            .zip(params.into_iter())
            .into_iter()
        {
            args.push(
                st.cascade(env, arg)?
            );
        }
        Ok(args)
    }
}

fn function_conversion_rules(env: &Box<Environment>, sequence_type: Option<SequenceType>, argument: Object) -> Result<Object, ErrorInfo> {
    if let Some(st) = sequence_type {
        match argument {
            Object::Atomic(Type::Untyped(_)) => {
                if st.item_type.is_same(env, &ItemType::AtomicOrUnionType(XS_QNAME.into()))
                    || st.item_type.is_same(env, &ItemType::AtomicOrUnionType(XS_NOTATION.into())) {
                    return Err((ErrorCode::XPTY0117, String::from("TODO")))
                }
            }
            _ => {}
        }
        Ok(st.cascade(env, argument)?)
    } else {
        Ok(argument)
    }
}

pub(crate) fn call(env: Box<Environment>, name: QNameResolved, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    println!("call: {:?} {:?}", name, arguments);
    let mut fn_env = env.next();

    let fun = fn_env.declared_functions(&name, arguments.len());
    if fun.is_some() {
        let fun = fun.unwrap().clone();

        for (parameter, mut argument) in (&fun.parameters).into_iter()
            .zip(arguments.into_iter())
            .into_iter()
        {
            argument = function_conversion_rules(&fn_env, parameter.sequence_type.clone(), argument)?;

            fn_env.set_variable(resolve_element_qname(&parameter.name, &fn_env), argument.clone())
        }

        let (new_env, result) = fun.body.eval(fn_env, context)?;
        let env = new_env.prev();

        Ok((env, result))

    } else {
        // workaround for concat function
        let fun: Option<FUNCTION> = if name.local_part == "concat" {
            if name.url == *XPATH_FUNCTIONS.uri && arguments.len() >= 2 {
                Some(strings::FN_CONCAT(arguments.len()))
            } else {
                None
            }
        } else {
            fn_env.get_function(&name, arguments.len())
        };

        if let Some(((params, st), body)) = fun {

            let mut checked_arguments = Vec::with_capacity(arguments.len());
            for (parameter, mut argument) in (&params).into_iter()
                .zip(arguments.into_iter())
                .into_iter()
            {
                argument = function_conversion_rules(&fn_env, Some(parameter.clone()), argument)?;

                checked_arguments.push(argument);
            }

            let (new_env, mut result) = body(fn_env, checked_arguments, context)?;
            result = st.cascade(&new_env, result)?;

            let env = new_env.prev();

            Ok((env, result))
        } else {
            Err((ErrorCode::XPST0017, format!("no function {:?}#{:?}", name, arguments.len())))
        }
    }
}