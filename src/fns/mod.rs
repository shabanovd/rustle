use std::collections::HashMap;
use crate::eval::{Object, Type};
use crate::eval::Environment;
use crate::namespaces::*;
use crate::parser::Expr;
use crate::value::{QName, QNameResolved};

mod decimal;
mod url;
mod map;
mod fun;

pub type FUNCTION<'a> = fn(Box<Environment<'a>>, Vec<Object>, &Object) -> (Box<Environment<'a>>, Object);

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: QName,
    pub sequenceType: Option<Type> // TODO: new type?
}

#[derive(Clone)]
pub struct FunctionsRegister<'a> {
    functions: HashMap<QNameResolved, HashMap<usize, FUNCTION<'a>>>,
}

impl<'a> FunctionsRegister<'a> {
    pub fn new() -> Self {
        let mut instance = FunctionsRegister {
            functions: HashMap::new(),
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

        instance
    }

    pub fn register(&mut self, url: &str, local_part: &str, arity: usize, fun: FUNCTION<'a>) {
        self.functions.entry(QNameResolved { url: String::from(url), local_part: String::from(local_part) })
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
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

    pub fn eval(&self, env: Box<Environment<'a>>, url: &str, local_part: &str, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
        let qname = QNameResolved { url: String::from(url), local_part: String::from(local_part) };

        println!("eval_builtin: {:?} {:?}", qname, arguments);

        let fun: Option<FUNCTION> = env.functions.get(&qname, arguments.len());

        if fun.is_some() {
            fun.unwrap()(env, arguments, context_item)
        } else {
            (env, Object::Empty)
        }
    }
}