use std::collections::HashMap;
use crate::eval::{Object, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::namespaces::*;
use crate::values::{QName, QNameResolved, resolve_element_qname};

mod fun;
mod sequences;
mod qname;
mod boolean;
mod strings;
mod types;
mod datetime;
mod nodes;
mod comparison;
mod math;
mod map;
mod array;
mod aggregates;

pub use crate::fns::boolean::object_to_bool;

use crate::parser::errors::ErrorCode;
use crate::eval::expression::Expression;
use crate::eval::sequence_type::SequenceType;

pub type FUNCTION<'a> = fn(Box<Environment<'a>>, Vec<Object>, &DynamicContext) -> EvalResult<'a>;

#[derive(Clone)]
pub struct Function {
    name: QNameResolved,
    parameters: Vec<Param>,
    body: Box<dyn Expression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct FunctionsRegister<'a> {
    functions: HashMap<QNameResolved, HashMap<usize, FUNCTION<'a>>>,
    declared: HashMap<QNameResolved, HashMap<usize, Function>>,
}

impl<'a> FunctionsRegister<'a> {
    pub(crate) fn new() -> Self {
        let mut instance = FunctionsRegister {
            functions: HashMap::new(),
            declared: HashMap::new(),
        };

        instance.register(SCHEMA.url, "untypedAtomic", 1, types::xs_untyped_atomic_eval);
        instance.register(SCHEMA.url, "boolean", 1, types::xs_boolean_eval);
        instance.register(SCHEMA.url, "string", 1, types::xs_string_eval);
        instance.register(SCHEMA.url, "NCName", 1, types::xs_ncname_eval);
        instance.register(SCHEMA.url, "anyURI", 1, types::xs_anyuri_eval);
        instance.register(SCHEMA.url, "date", 1, types::xs_date_eval);
        instance.register(SCHEMA.url, "dateTime", 1, types::xs_date_time_eval);
        instance.register(SCHEMA.url, "yearMonthDuration", 1, types::xs_year_month_duration_eval);
        instance.register(SCHEMA.url, "dayTimeDuration", 1, types::xs_day_time_duration_eval);
        instance.register(SCHEMA.url, "duration", 1, types::xs_duration_eval);

        instance.register(SCHEMA.url, "integer", 1, types::xs_integer_eval);
        instance.register(SCHEMA.url, "decimal", 1, types::xs_decimal_eval);
        instance.register(SCHEMA.url, "float", 1, types::xs_float_eval);
        instance.register(SCHEMA.url, "double", 1, types::xs_double_eval);

        instance.register(SCHEMA.url, "nonPositiveInteger", 1, types::xs_non_positive_integer_eval);
        instance.register(SCHEMA.url, "negativeInteger", 1, types::xs_negative_integer_eval);
        instance.register(SCHEMA.url, "long", 1, types::xs_long_eval);
        instance.register(SCHEMA.url, "int", 1, types::xs_int_eval);
        instance.register(SCHEMA.url, "short", 1, types::xs_short_eval);
        instance.register(SCHEMA.url, "byte", 1, types::xs_byte_eval);
        instance.register(SCHEMA.url, "nonNegativeInteger", 1, types::xs_non_negative_integer_eval);
        instance.register(SCHEMA.url, "unsignedLong", 1, types::xs_unsigned_long_eval);
        instance.register(SCHEMA.url, "unsignedInt", 1, types::xs_unsigned_int_eval);
        instance.register(SCHEMA.url, "unsignedShort", 1, types::xs_unsigned_short_eval);
        instance.register(SCHEMA.url, "unsignedByte", 1, types::xs_unsigned_byte_eval);
        instance.register(SCHEMA.url, "positiveInteger", 1, types::xs_positive_integer_eval);

        instance.register(XPATH_FUNCTIONS.url, "resolve-QName", 2, qname::fn_resolve_qname);
        instance.register(XPATH_FUNCTIONS.url, "QName", 2, qname::fn_qname);

//        instance.register("op", "same-key", 2, map::map_merge);
        instance.register(XPATH_MAP.url, "merge", 1, map::map_merge);
        instance.register(XPATH_MAP.url, "merge", 2, map::map_merge);
        instance.register(XPATH_MAP.url, "size", 1, map::map_size);
        instance.register(XPATH_MAP.url, "contains", 2, map::map_contains);
        instance.register(XPATH_MAP.url, "get", 2, map::map_get);
        instance.register(XPATH_MAP.url, "find", 2, map::map_find);
        instance.register(XPATH_MAP.url, "put", 3, map::map_put);
        instance.register(XPATH_MAP.url, "find", 2, map::map_find);
        instance.register(XPATH_MAP.url, "entry", 2, map::map_entry);
        instance.register(XPATH_MAP.url, "remove", 2, map::map_remove);
        instance.register(XPATH_MAP.url, "for-each", 2, map::map_for_each);

        instance.register(XPATH_ARRAY.url, "size", 1, array::size);
        instance.register(XPATH_ARRAY.url, "get", 2, array::get);
        instance.register(XPATH_ARRAY.url, "put", 3, array::put);
        instance.register(XPATH_ARRAY.url, "append", 2, array::append);
        instance.register(XPATH_ARRAY.url, "subarray", 2, array::subarray);
        instance.register(XPATH_ARRAY.url, "subarray", 3, array::subarray);
        instance.register(XPATH_ARRAY.url, "insert-before", 3, array::insert_before);
        instance.register(XPATH_ARRAY.url, "head", 1, array::head);
        instance.register(XPATH_ARRAY.url, "tail", 1, array::tail);
        instance.register(XPATH_ARRAY.url, "reverse", 1, array::reverse);
        instance.register(XPATH_ARRAY.url, "join", usize::MAX, array::join);
        instance.register(XPATH_ARRAY.url, "for-each", 2, array::for_each);
        instance.register(XPATH_ARRAY.url, "filter", 2, array::filter);
        instance.register(XPATH_ARRAY.url, "fold-left", 3, array::fold_left);
        instance.register(XPATH_ARRAY.url, "fold-right", 3, array::fold_right);
        instance.register(XPATH_ARRAY.url, "for-each-pair", 3, array::for_each_pair);
        instance.register(XPATH_ARRAY.url, "sort", 1, array::sort);
        instance.register(XPATH_ARRAY.url, "sort", 2, array::sort);
        instance.register(XPATH_ARRAY.url, "sort", 3, array::sort);
        instance.register(XPATH_ARRAY.url, "flatten", 1, array::flatten);

        instance.register(XPATH_FUNCTIONS.url, "current-date", 0, datetime::fn_current_date);
        instance.register(XPATH_FUNCTIONS.url, "current-time", 0, datetime::fn_current_time);
        instance.register(XPATH_FUNCTIONS.url, "year-from-date", 1, datetime::fn_year_from_date);
        instance.register(XPATH_FUNCTIONS.url, "month-from-date", 1, datetime::fn_month_from_date);
        instance.register(XPATH_FUNCTIONS.url, "day-from-date", 1, datetime::fn_day_from_date);
        instance.register(XPATH_FUNCTIONS.url, "days-from-duration", 1, datetime::fn_days_from_duration);

        instance.register(XPATH_FUNCTIONS.url, "timezone-from-time", 1, datetime::fn_timezone_from_time);

        instance.register(XPATH_FUNCTIONS.url, "for-each", 2, fun::for_each);
        instance.register(XPATH_FUNCTIONS.url, "filter", 2, fun::filter);
        instance.register(XPATH_FUNCTIONS.url, "fold-left", 3, fun::fold_left);
        instance.register(XPATH_FUNCTIONS.url, "fold-right", 3, fun::fold_right);
        instance.register(XPATH_FUNCTIONS.url, "for-each-pair", 3, fun::for_each_pair);
        instance.register(XPATH_FUNCTIONS.url, "sort", 1, fun::sort);
        instance.register(XPATH_FUNCTIONS.url, "sort", 2, fun::sort);
        instance.register(XPATH_FUNCTIONS.url, "sort", 3, fun::sort);
        instance.register(XPATH_FUNCTIONS.url, "apply", 2, fun::apply);

        instance.register(XPATH_FUNCTIONS.url, "error", 0, fun::error);
        instance.register(XPATH_FUNCTIONS.url, "error", 1, fun::error);
        instance.register(XPATH_FUNCTIONS.url, "error", 2, fun::error);
        instance.register(XPATH_FUNCTIONS.url, "error", 3, fun::error);

        instance.register(XPATH_FUNCTIONS.url, "count", 1, aggregates::fn_count);
        instance.register(XPATH_FUNCTIONS.url, "avg", 1, aggregates::fn_avg);
        instance.register(XPATH_FUNCTIONS.url, "max", 1, aggregates::fn_max);
        instance.register(XPATH_FUNCTIONS.url, "max", 2, aggregates::fn_max);
        instance.register(XPATH_FUNCTIONS.url, "min", 1, aggregates::fn_min);
        instance.register(XPATH_FUNCTIONS.url, "min", 2, aggregates::fn_min);
        // instance.register(XPATH_FUNCTIONS.url, "sum", 1, aggregates::fn_sum);
        // instance.register(XPATH_FUNCTIONS.url, "sum", 2, aggregates::fn_sum);

        instance.register(XPATH_FUNCTIONS.url, "abs", 1, math::fn_abs);
        instance.register(XPATH_FUNCTIONS.url, "floor", 1, math::fn_floor);
        instance.register(XPATH_FUNCTIONS.url, "round", 1, math::fn_round);
        instance.register(XPATH_FUNCTIONS.url, "round", 2, math::fn_round);
        instance.register(XPATH_FUNCTIONS.url, "round-half-to-even", 1, math::fn_round_half_to_even);
        instance.register(XPATH_FUNCTIONS.url, "round-half-to-even", 2, math::fn_round_half_to_even);

        instance.register(XPATH_FUNCTIONS.url, "boolean", 1, boolean::fn_boolean);
        instance.register(XPATH_FUNCTIONS.url, "true", 0, boolean::fn_true);
        instance.register(XPATH_FUNCTIONS.url, "false", 0, boolean::fn_false);
        instance.register(XPATH_FUNCTIONS.url, "not", 1, boolean::fn_not);

        instance.register(XPATH_FUNCTIONS.url, "string", 0, strings::fn_string);
        instance.register(XPATH_FUNCTIONS.url, "string", 1, strings::fn_string);
        instance.register(XPATH_FUNCTIONS.url, "concat", 2, strings::fn_concat); // TODO number of arguments 2 or more
        instance.register(XPATH_FUNCTIONS.url, "concat", 3, strings::fn_concat);
        instance.register(XPATH_FUNCTIONS.url, "string-join", 1, strings::fn_string_join);
        instance.register(XPATH_FUNCTIONS.url, "string-join", 2, strings::fn_string_join);
        instance.register(XPATH_FUNCTIONS.url, "string-length", 0, strings::fn_string_length);
        instance.register(XPATH_FUNCTIONS.url, "string-length", 1, strings::fn_string_length);
        instance.register(XPATH_FUNCTIONS.url, "upper-case", 1, strings::fn_upper_case);
        instance.register(XPATH_FUNCTIONS.url, "lower-case", 1, strings::fn_lower_case);
        instance.register(XPATH_FUNCTIONS.url, "string-to-codepoints", 1, strings::fn_string_to_codepoints);

        instance.register(XPATH_FUNCTIONS.url, "starts-with", 2, strings::fn_starts_with);
        instance.register(XPATH_FUNCTIONS.url, "starts-with", 3, strings::fn_starts_with);
        instance.register(XPATH_FUNCTIONS.url, "ends-with", 2, strings::fn_ends_with);
        instance.register(XPATH_FUNCTIONS.url, "ends-with", 3, strings::fn_ends_with);

        instance.register(XPATH_FUNCTIONS.url, "position", 0, sequences::fn_position);
        instance.register(XPATH_FUNCTIONS.url, "last", 0, sequences::fn_last);

        instance.register(XPATH_FUNCTIONS.url, "data", 0, sequences::fn_data);
        instance.register(XPATH_FUNCTIONS.url, "data", 1, sequences::fn_data);
        instance.register(XPATH_FUNCTIONS.url, "empty", 1, sequences::fn_empty);
        instance.register(XPATH_FUNCTIONS.url, "remove", 2, sequences::fn_remove);
        instance.register(XPATH_FUNCTIONS.url, "reverse", 1, sequences::fn_reverse);
        instance.register(XPATH_FUNCTIONS.url, "subsequence", 2, sequences::fn_subsequence);
        instance.register(XPATH_FUNCTIONS.url, "subsequence", 3, sequences::fn_subsequence);

        instance.register(XPATH_FUNCTIONS.url, "zero-or-one", 1, sequences::fn_zero_or_one);
        instance.register(XPATH_FUNCTIONS.url, "one-or-more", 1, sequences::fn_one_or_more);
        instance.register(XPATH_FUNCTIONS.url, "exactly-one", 1, sequences::fn_exactly_one);

        instance.register(XPATH_FUNCTIONS.url, "deep-equal", 2, comparison::fn_deep_equal);

        instance.register(XPATH_FUNCTIONS.url, "name", 0, nodes::fn_name);
        instance.register(XPATH_FUNCTIONS.url, "name", 1, nodes::fn_name);
        instance.register(XPATH_FUNCTIONS.url, "local-name", 0, nodes::fn_local_name);
        instance.register(XPATH_FUNCTIONS.url, "local-name", 1, nodes::fn_local_name);
        instance.register(XPATH_FUNCTIONS.url, "namespace-uri", 0, nodes::fn_namespace_uri);
        instance.register(XPATH_FUNCTIONS.url, "namespace-uri", 1, nodes::fn_namespace_uri);
        instance.register(XPATH_FUNCTIONS.url, "lang", 1, nodes::fn_lang);
        instance.register(XPATH_FUNCTIONS.url, "lang", 2, nodes::fn_lang);
        instance.register(XPATH_FUNCTIONS.url, "root", 0, nodes::fn_root);
        instance.register(XPATH_FUNCTIONS.url, "root", 1, nodes::fn_root);
        instance.register(XPATH_FUNCTIONS.url, "path", 0, nodes::fn_path);
        instance.register(XPATH_FUNCTIONS.url, "path", 1, nodes::fn_path);
        instance.register(XPATH_FUNCTIONS.url, "has-children", 0, nodes::fn_has_children);
        instance.register(XPATH_FUNCTIONS.url, "has-children", 1, nodes::fn_has_children);
        instance.register(XPATH_FUNCTIONS.url, "innermost", 1, nodes::fn_innermost);
        instance.register(XPATH_FUNCTIONS.url, "outermost", 1, nodes::fn_outermost);

        instance
    }

    pub(crate) fn register(&mut self, url: &str, local_part: &str, arity: usize, fun: FUNCTION<'a>) {
        self.functions.entry(QNameResolved { url: String::from(url), local_part: String::from(local_part) })
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
    }

    pub(crate) fn put(&mut self, name: QNameResolved, parameters: Vec<Param>, body: Box<dyn Expression>) {
        self.declared.entry(name.clone())
            .or_insert_with(HashMap::new)
            .insert(parameters.len(), Function { name, parameters, body });
    }

    pub(crate) fn get(&self, qname: &QNameResolved, arity: usize) -> Option<FUNCTION<'a>> {
        if let Some(list) = self.functions.get(qname) {
            if let Some(rf) = list.get(&arity) {
                Some(*rf)
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

pub(crate) fn call<'a>(env: Box<Environment<'a>>, name: QNameResolved, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    // println!("call: {:?} {:?}", name, arguments);

    let fun = env.functions.declared(&name, arguments.len());
    if fun.is_some() {
        let fun = fun.unwrap();

        let mut fn_env = Environment::new();
        fun.parameters.clone().into_iter()
            .zip(arguments.into_iter())
            .for_each(
                |(parameter, argument)|
                    fn_env.set(resolve_element_qname(&parameter.name, &env), argument.clone())
            );

        let (_, result) = fun.body.eval(Box::new(fn_env), context)?;

        Ok((env, result))

    } else {
        let fun: Option<FUNCTION> = env.functions.get(&name, arguments.len());

        if fun.is_some() {
            fun.unwrap()(env, arguments, context)
        } else {
            Err((ErrorCode::XPST0017, format!("no function {:?}#{:?}", name, arguments.len())))
        }
    }
}