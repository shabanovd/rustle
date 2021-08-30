use std::collections::HashMap;
use crate::eval::{Object, Type, eval_expr};
use crate::eval::Environment;
use crate::namespaces::*;
use crate::parser::Expr;
use crate::value::{QName, QNameResolved, resolve_element_qname};

mod fun;
mod sequences;
mod boolean;
mod strings;
mod decimal;
mod math;
mod url;
mod map;

pub use strings::object_to_string;

pub type FUNCTION<'a> = fn(Box<Environment<'a>>, Vec<Object>, &Object) -> (Box<Environment<'a>>, Object);

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    name: QNameResolved,
    parameters: Vec<Param>,
    body: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: QName,
    pub sequence_type: Option<Type> // TODO: new type?
}

#[derive(Clone)]
pub struct FunctionsRegister<'a> {
    functions: HashMap<QNameResolved, HashMap<usize, FUNCTION<'a>>>,
    declared: HashMap<QNameResolved, HashMap<usize, Function>>,
}

impl<'a> FunctionsRegister<'a> {
    pub fn new() -> Self {
        let mut instance = FunctionsRegister {
            functions: HashMap::new(),
            declared: HashMap::new(),
        };

        instance.register(SCHEMA.url, "decimal", 1, decimal::xs_decimal_eval);
        instance.register(SCHEMA.url, "anyURI", 1, url::xs_anyuri_eval);

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

        instance.register(XPATH_FUNCTIONS.url, "apply", 2, fun::apply);

        instance.register(XPATH_FUNCTIONS.url, "abs", 1, math::fn_abs);


        instance.register(XPATH_FUNCTIONS.url, "true", 0, boolean::fn_true);
        instance.register(XPATH_FUNCTIONS.url, "false", 0, boolean::fn_false);

        instance.register(XPATH_FUNCTIONS.url, "string", 0, strings::fn_string);
        instance.register(XPATH_FUNCTIONS.url, "string", 1, strings::fn_string);
        instance.register(XPATH_FUNCTIONS.url, "string-join", 1, strings::fn_string_join);
        instance.register(XPATH_FUNCTIONS.url, "string-join", 2, strings::fn_string_join);


        instance.register(XPATH_FUNCTIONS.url, "empty", 1, sequences::fn_empty);
        instance.register(XPATH_FUNCTIONS.url, "reverse", 1, sequences::fn_reverse);

        instance
    }

    pub fn register(&mut self, url: &str, local_part: &str, arity: usize, fun: FUNCTION<'a>) {
        self.functions.entry(QNameResolved { url: String::from(url), local_part: String::from(local_part) })
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
    }

    pub fn put(&mut self, name: QNameResolved, parameters: Vec<Param>, body: Box<Expr>) {
        self.declared.entry(name.clone())
            .or_insert_with(HashMap::new)
            .insert(parameters.len(), Function { name, parameters, body: *body });
    }

    pub fn get(&self, qname: &QNameResolved, arity: usize) -> Option<FUNCTION<'a>> {
        // println!("function get {:?} {:?} {:?}", uri, local_part, arity);
        if let Some(list) = self.functions.get(qname) {
            // println!("function list {:?}", list.len());
            //TODO: fix it!
            let rf = list.get(&arity).unwrap();
            Some(*rf)
        } else {
            // println!("function list NONE");
            None
        }
    }

    pub fn declared(&self, qname: &QNameResolved, arity: usize) -> Option<Function> {
        // println!("function get {:?} {:?} {:?}", uri, local_part, arity);
        if let Some(list) = self.declared.get(qname) {
            // println!("function list {:?}", list.len());
            //TODO: fix it!
            let rf = list.get(&arity).unwrap();
            Some(rf.clone())
        } else {
            // println!("function list NONE");
            None
        }
    }
}

pub fn expr_to_params(expr: Expr) -> Vec<Param> {
    match expr {
        Expr::ParamList(exprs) => {
            let mut params = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let param = match expr {
                    Expr::Param { name, type_declaration } => {
                        Param { name, sequence_type: None }
                    }
                    _ => panic!("expected Param but got {:?}", expr)
                };
                params.push(param);
            }
            params
        },
        _ => panic!("expected ParamList but got {:?}", expr)
    }
}

pub fn call<'a>(env: Box<Environment<'a>>, name: QNameResolved, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    println!("call: {:?} {:?}", name, arguments);

    let fun = env.functions.declared(&name, arguments.len());
    if fun.is_some() {
        let fun = fun.unwrap();

        let mut fn_env = Environment::new();
        fun.parameters.into_iter()
            .zip(arguments.into_iter())
            .for_each(
                |(parameter, argument)|
                    fn_env.set(resolve_element_qname(parameter.name, &env), argument.clone())
            );

        let (_, result) = eval_expr(fun.body.clone(), Box::new(fn_env), context_item);

        (env, result)

    } else {
        let fun: Option<FUNCTION> = env.functions.get(&name, arguments.len());

        if fun.is_some() {
            fun.unwrap()(env, arguments, context_item)
        } else {
            panic!("no function {:?}#{:?}", name, arguments.len())
        }
    }
}