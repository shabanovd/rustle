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
mod context;
mod comparison;
mod math;
mod map;
mod array;
mod aggregates;

pub use crate::fns::boolean::object_to_bool;

use crate::parser::errors::ErrorCode;
use crate::eval::expression::Expression;
use crate::eval::sequence_type::SequenceType;

pub type FUNCTION = fn(Box<Environment>, Vec<Object>, &DynamicContext) -> EvalResult;

#[derive(Clone)]
pub struct Function {
    name: QNameResolved,
    parameters: Vec<Param>,
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

        instance.register(&*SCHEMA.uri, "untypedAtomic", 1, types::xs_untyped_atomic_eval);
        instance.register(&*SCHEMA.uri, "boolean", 1, types::xs_boolean_eval);
        instance.register(&*SCHEMA.uri, "string", 1, types::xs_string_eval);
        instance.register(&*SCHEMA.uri, "NCName", 1, types::xs_ncname_eval);
        instance.register(&*SCHEMA.uri, "anyURI", 1, types::xs_anyuri_eval);
        instance.register(&*SCHEMA.uri, "date", 1, types::xs_date_eval);
        instance.register(&*SCHEMA.uri, "dateTime", 1, types::xs_date_time_eval);
        instance.register(&*SCHEMA.uri, "yearMonthDuration", 1, types::xs_year_month_duration_eval);
        instance.register(&*SCHEMA.uri, "dayTimeDuration", 1, types::xs_day_time_duration_eval);
        instance.register(&*SCHEMA.uri, "duration", 1, types::xs_duration_eval);

        instance.register(&*SCHEMA.uri, "hexBinary", 1, types::xs_hex_binary_eval);

        instance.register(&*SCHEMA.uri, "integer", 1, types::xs_integer_eval);
        instance.register(&*SCHEMA.uri, "decimal", 1, types::xs_decimal_eval);
        instance.register(&*SCHEMA.uri, "float", 1, types::xs_float_eval);
        instance.register(&*SCHEMA.uri, "double", 1, types::xs_double_eval);

        instance.register(&*SCHEMA.uri, "nonPositiveInteger", 1, types::xs_non_positive_integer_eval);
        instance.register(&*SCHEMA.uri, "negativeInteger", 1, types::xs_negative_integer_eval);
        instance.register(&*SCHEMA.uri, "long", 1, types::xs_long_eval);
        instance.register(&*SCHEMA.uri, "int", 1, types::xs_int_eval);
        instance.register(&*SCHEMA.uri, "short", 1, types::xs_short_eval);
        instance.register(&*SCHEMA.uri, "byte", 1, types::xs_byte_eval);
        instance.register(&*SCHEMA.uri, "nonNegativeInteger", 1, types::xs_non_negative_integer_eval);
        instance.register(&*SCHEMA.uri, "unsignedLong", 1, types::xs_unsigned_long_eval);
        instance.register(&*SCHEMA.uri, "unsignedInt", 1, types::xs_unsigned_int_eval);
        instance.register(&*SCHEMA.uri, "unsignedShort", 1, types::xs_unsigned_short_eval);
        instance.register(&*SCHEMA.uri, "unsignedByte", 1, types::xs_unsigned_byte_eval);
        instance.register(&*SCHEMA.uri, "positiveInteger", 1, types::xs_positive_integer_eval);

        instance.register(&*XPATH_FUNCTIONS.uri, "resolve-QName", 2, qname::fn_resolve_qname);
        instance.register(&*XPATH_FUNCTIONS.uri, "QName", 2, qname::fn_qname);
        instance.register(&*XPATH_FUNCTIONS.uri, "prefix-from-QName", 1, qname::fn_prefix_from_qname);
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name-from-QName", 1, qname::fn_local_name_from_qname);
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri-from-QName", 1, qname::fn_namespace_uri_from_qname);
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri-for-prefix", 1, qname::fn_namespace_uri_for_prefix);
        instance.register(&*XPATH_FUNCTIONS.uri, "in-scope-prefixes", 1, qname::fn_in_scope_prefixes);

        instance.register(&*XPATH_FUNCTIONS.uri, "node-name", 0, qname::fn_node_name);
        instance.register(&*XPATH_FUNCTIONS.uri, "node-name", 1, qname::fn_node_name);


//        instance.register("op", "same-key", 2, map::map_merge);
        instance.register(&*XPATH_MAP.uri, "merge", 1, map::map_merge);
        instance.register(&*XPATH_MAP.uri, "merge", 2, map::map_merge);
        instance.register(&*XPATH_MAP.uri, "size", 1, map::map_size);
        instance.register(&*XPATH_MAP.uri, "contains", 2, map::map_contains);
        instance.register(&*XPATH_MAP.uri, "get", 2, map::map_get);
        instance.register(&*XPATH_MAP.uri, "find", 2, map::map_find);
        instance.register(&*XPATH_MAP.uri, "put", 3, map::map_put);
        instance.register(&*XPATH_MAP.uri, "find", 2, map::map_find);
        instance.register(&*XPATH_MAP.uri, "entry", 2, map::map_entry);
        instance.register(&*XPATH_MAP.uri, "remove", 2, map::map_remove);
        instance.register(&*XPATH_MAP.uri, "for-each", 2, map::map_for_each);

        instance.register(&*XPATH_ARRAY.uri, "size", 1, array::size);
        instance.register(&*XPATH_ARRAY.uri, "get", 2, array::get);
        instance.register(&*XPATH_ARRAY.uri, "put", 3, array::put);
        instance.register(&*XPATH_ARRAY.uri, "append", 2, array::append);
        instance.register(&*XPATH_ARRAY.uri, "subarray", 2, array::subarray);
        instance.register(&*XPATH_ARRAY.uri, "subarray", 3, array::subarray);
        instance.register(&*XPATH_ARRAY.uri, "insert-before", 3, array::insert_before);
        instance.register(&*XPATH_ARRAY.uri, "head", 1, array::head);
        instance.register(&*XPATH_ARRAY.uri, "tail", 1, array::tail);
        instance.register(&*XPATH_ARRAY.uri, "reverse", 1, array::reverse);
        instance.register(&*XPATH_ARRAY.uri, "join", usize::MAX, array::join);
        instance.register(&*XPATH_ARRAY.uri, "for-each", 2, array::for_each);
        instance.register(&*XPATH_ARRAY.uri, "filter", 2, array::filter);
        instance.register(&*XPATH_ARRAY.uri, "fold-left", 3, array::fold_left);
        instance.register(&*XPATH_ARRAY.uri, "fold-right", 3, array::fold_right);
        instance.register(&*XPATH_ARRAY.uri, "for-each-pair", 3, array::for_each_pair);
        instance.register(&*XPATH_ARRAY.uri, "sort", 1, array::sort);
        instance.register(&*XPATH_ARRAY.uri, "sort", 2, array::sort);
        instance.register(&*XPATH_ARRAY.uri, "sort", 3, array::sort);
        instance.register(&*XPATH_ARRAY.uri, "flatten", 1, array::flatten);

        instance.register(&*XPATH_FUNCTIONS.uri, "current-date", 0, datetime::fn_current_date);
        instance.register(&*XPATH_FUNCTIONS.uri, "current-time", 0, datetime::fn_current_time);
        instance.register(&*XPATH_FUNCTIONS.uri, "year-from-date", 1, datetime::fn_year_from_date);
        instance.register(&*XPATH_FUNCTIONS.uri, "month-from-date", 1, datetime::fn_month_from_date);
        instance.register(&*XPATH_FUNCTIONS.uri, "day-from-date", 1, datetime::fn_day_from_date);
        instance.register(&*XPATH_FUNCTIONS.uri, "days-from-duration", 1, datetime::fn_days_from_duration);

        instance.register(&*XPATH_FUNCTIONS.uri, "timezone-from-time", 1, datetime::fn_timezone_from_time);

        instance.register(&*XPATH_FUNCTIONS.uri, "for-each", 2, fun::for_each);
        instance.register(&*XPATH_FUNCTIONS.uri, "filter", 2, fun::filter);
        instance.register(&*XPATH_FUNCTIONS.uri, "fold-left", 3, fun::fold_left);
        instance.register(&*XPATH_FUNCTIONS.uri, "fold-right", 3, fun::fold_right);
        instance.register(&*XPATH_FUNCTIONS.uri, "for-each-pair", 3, fun::for_each_pair);
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 1, fun::sort);
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 2, fun::sort);
        instance.register(&*XPATH_FUNCTIONS.uri, "sort", 3, fun::sort);
        instance.register(&*XPATH_FUNCTIONS.uri, "apply", 2, fun::apply);

        instance.register(&*XPATH_FUNCTIONS.uri, "error", 0, fun::error);
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 1, fun::error);
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 2, fun::error);
        instance.register(&*XPATH_FUNCTIONS.uri, "error", 3, fun::error);

        instance.register(&*XPATH_FUNCTIONS.uri, "format-number", 2, math::fn_format_number_eval);
        instance.register(&*XPATH_FUNCTIONS.uri, "format-number", 3, math::fn_format_number_eval);

        instance.register(&*XPATH_FUNCTIONS.uri, "number", 0, math::fn_number_eval);
        instance.register(&*XPATH_FUNCTIONS.uri, "number", 1, math::fn_number_eval);

        instance.register(&*XPATH_FUNCTIONS.uri, "count", 1, aggregates::fn_count);
        instance.register(&*XPATH_FUNCTIONS.uri, "avg", 1, aggregates::fn_avg);
        instance.register(&*XPATH_FUNCTIONS.uri, "max", 1, aggregates::fn_max);
        instance.register(&*XPATH_FUNCTIONS.uri, "max", 2, aggregates::fn_max);
        instance.register(&*XPATH_FUNCTIONS.uri, "min", 1, aggregates::fn_min);
        instance.register(&*XPATH_FUNCTIONS.uri, "min", 2, aggregates::fn_min);
        // instance.register(&*XPATH_FUNCTIONS.url, "sum", 1, aggregates::fn_sum);
        // instance.register(&*XPATH_FUNCTIONS.url, "sum", 2, aggregates::fn_sum);

        instance.register(&*XPATH_MATH.uri, "pi", 0, math::fn_pi);

        instance.register(&*XPATH_FUNCTIONS.uri, "abs", 1, math::fn_abs);
        instance.register(&*XPATH_FUNCTIONS.uri, "floor", 1, math::fn_floor);
        instance.register(&*XPATH_FUNCTIONS.uri, "round", 1, math::fn_round);
        instance.register(&*XPATH_FUNCTIONS.uri, "round", 2, math::fn_round);
        instance.register(&*XPATH_FUNCTIONS.uri, "round-half-to-even", 1, math::fn_round_half_to_even);
        instance.register(&*XPATH_FUNCTIONS.uri, "round-half-to-even", 2, math::fn_round_half_to_even);

        instance.register(&*XPATH_FUNCTIONS.uri, "boolean", 1, boolean::fn_boolean);
        instance.register(&*XPATH_FUNCTIONS.uri, "true", 0, boolean::fn_true);
        instance.register(&*XPATH_FUNCTIONS.uri, "false", 0, boolean::fn_false);
        instance.register(&*XPATH_FUNCTIONS.uri, "not", 1, boolean::fn_not);

        instance.register(&*XPATH_FUNCTIONS.uri, "string", 0, strings::fn_string);
        instance.register(&*XPATH_FUNCTIONS.uri, "string", 1, strings::fn_string);
        instance.register(&*XPATH_FUNCTIONS.uri, "concat", 2, strings::fn_concat); // TODO number of arguments 2 or more
        instance.register(&*XPATH_FUNCTIONS.uri, "concat", 3, strings::fn_concat);
        instance.register(&*XPATH_FUNCTIONS.uri, "string-join", 1, strings::fn_string_join);
        instance.register(&*XPATH_FUNCTIONS.uri, "string-join", 2, strings::fn_string_join);
        instance.register(&*XPATH_FUNCTIONS.uri, "string-length", 0, strings::fn_string_length);
        instance.register(&*XPATH_FUNCTIONS.uri, "string-length", 1, strings::fn_string_length);
        instance.register(&*XPATH_FUNCTIONS.uri, "normalize-space", 0, strings::fn_normalize_space);
        instance.register(&*XPATH_FUNCTIONS.uri, "normalize-space", 1, strings::fn_normalize_space);
        instance.register(&*XPATH_FUNCTIONS.uri, "upper-case", 1, strings::fn_upper_case);
        instance.register(&*XPATH_FUNCTIONS.uri, "lower-case", 1, strings::fn_lower_case);
        instance.register(&*XPATH_FUNCTIONS.uri, "string-to-codepoints", 1, strings::fn_string_to_codepoints);

        instance.register(&*XPATH_FUNCTIONS.uri, "starts-with", 2, strings::fn_starts_with);
        instance.register(&*XPATH_FUNCTIONS.uri, "starts-with", 3, strings::fn_starts_with);
        instance.register(&*XPATH_FUNCTIONS.uri, "ends-with", 2, strings::fn_ends_with);
        instance.register(&*XPATH_FUNCTIONS.uri, "ends-with", 3, strings::fn_ends_with);

        instance.register(&*XPATH_FUNCTIONS.uri, "position", 0, sequences::fn_position);
        instance.register(&*XPATH_FUNCTIONS.uri, "last", 0, sequences::fn_last);
        instance.register(&*XPATH_FUNCTIONS.uri, "default-collation", 0, context::fn_default_collation);
        instance.register(&*XPATH_FUNCTIONS.uri, "default-language", 0, context::fn_default_language);
        instance.register(&*XPATH_FUNCTIONS.uri, "static-base-uri", 0, context::fn_static_base_uri);

        instance.register(&*XPATH_FUNCTIONS.uri, "data", 0, sequences::fn_data);
        instance.register(&*XPATH_FUNCTIONS.uri, "data", 1, sequences::fn_data);
        instance.register(&*XPATH_FUNCTIONS.uri, "empty", 1, sequences::fn_empty);
        instance.register(&*XPATH_FUNCTIONS.uri, "remove", 2, sequences::fn_remove);
        instance.register(&*XPATH_FUNCTIONS.uri, "reverse", 1, sequences::fn_reverse);
        instance.register(&*XPATH_FUNCTIONS.uri, "subsequence", 2, sequences::fn_subsequence);
        instance.register(&*XPATH_FUNCTIONS.uri, "subsequence", 3, sequences::fn_subsequence);

        instance.register(&*XPATH_FUNCTIONS.uri, "zero-or-one", 1, sequences::fn_zero_or_one);
        instance.register(&*XPATH_FUNCTIONS.uri, "one-or-more", 1, sequences::fn_one_or_more);
        instance.register(&*XPATH_FUNCTIONS.uri, "exactly-one", 1, sequences::fn_exactly_one);

        instance.register(&*XPATH_FUNCTIONS.uri, "deep-equal", 2, comparison::fn_deep_equal);

        instance.register(&*XPATH_FUNCTIONS.uri, "name", 0, nodes::fn_name);
        instance.register(&*XPATH_FUNCTIONS.uri, "name", 1, nodes::fn_name);
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name", 0, nodes::fn_local_name);
        instance.register(&*XPATH_FUNCTIONS.uri, "local-name", 1, nodes::fn_local_name);
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri", 0, nodes::fn_namespace_uri);
        instance.register(&*XPATH_FUNCTIONS.uri, "namespace-uri", 1, nodes::fn_namespace_uri);
        instance.register(&*XPATH_FUNCTIONS.uri, "lang", 1, nodes::fn_lang);
        instance.register(&*XPATH_FUNCTIONS.uri, "lang", 2, nodes::fn_lang);
        instance.register(&*XPATH_FUNCTIONS.uri, "root", 0, nodes::fn_root);
        instance.register(&*XPATH_FUNCTIONS.uri, "root", 1, nodes::fn_root);
        instance.register(&*XPATH_FUNCTIONS.uri, "path", 0, nodes::fn_path);
        instance.register(&*XPATH_FUNCTIONS.uri, "path", 1, nodes::fn_path);
        instance.register(&*XPATH_FUNCTIONS.uri, "has-children", 0, nodes::fn_has_children);
        instance.register(&*XPATH_FUNCTIONS.uri, "has-children", 1, nodes::fn_has_children);
        instance.register(&*XPATH_FUNCTIONS.uri, "innermost", 1, nodes::fn_innermost);
        instance.register(&*XPATH_FUNCTIONS.uri, "outermost", 1, nodes::fn_outermost);

        instance
    }

    pub(crate) fn register<S: Into<String>>(&mut self, uri: S, local_part: &str, arity: usize, fun: FUNCTION) {
        self.functions.entry(QNameResolved { url: uri.into(), local_part: local_part.into() })
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
    }

    pub(crate) fn put(&mut self, name: QNameResolved, parameters: Vec<Param>, body: Box<dyn Expression>) {
        self.declared.entry(name.clone())
            .or_insert_with(HashMap::new)
            .insert(parameters.len(), Function { name, parameters, body });
    }

    pub(crate) fn get(&self, qname: &QNameResolved, arity: usize) -> Option<FUNCTION> {
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

pub(crate) fn call<'a>(env: Box<Environment>, name: QNameResolved, arguments: Vec<Object>, context: &'a DynamicContext) -> EvalResult {
    // println!("call: {:?} {:?}", name, arguments);
    let mut fn_env = env.next();

    let fun = fn_env.declared_functions(&name, arguments.len());
    if fun.is_some() {
        let fun = fun.unwrap().clone();

        for (parameter, argument) in (&fun.parameters).into_iter()
            .zip(arguments.into_iter())
            .into_iter()
        {
            fn_env.set_variable(resolve_element_qname(&parameter.name, &fn_env), argument.clone())
        }

        let (new_env, result) = fun.body.eval(fn_env, context)?;
        let env = new_env.prev();

        Ok((env, result))

    } else {
        let fun: Option<FUNCTION> = fn_env.get_function(&name, arguments.len());

        if let Some(fun) = fun {
            let (new_env, result) = fun(fn_env, arguments, context)?;
            let env = new_env.prev();

            Ok((env, result))
        } else {
            Err((ErrorCode::XPST0017, format!("no function {:?}#{:?}", name, arguments.len())))
        }
    }
}