use std::collections::HashMap;
use crate::eval::Object;
use crate::eval::Environment;
use crate::namespaces::*;

mod decimal;
mod url;
mod map;

pub type FUNCTION<'a> = fn(&'a mut Environment<'a>, Vec<Object>) -> (&'a mut Environment<'a>, Object);

pub struct FunctionsRegister<'a> {
    functions: HashMap<String, HashMap<usize, FUNCTION<'a>>>,
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

        instance
    }

    fn key(&self, uri: &str, local_part: &str) -> String {
        // possible optimization [uri, local_part].concat()
        format!("{{{}}}{}", uri, local_part)
    }

    pub fn register(&mut self, uri: &str, local_part: &str, arity: usize, fun: FUNCTION<'a>) {
        self.functions.entry(self.key(uri, local_part))
            .or_insert_with(HashMap::new)
            .insert(arity,fun);
    }

    pub fn get(&self, uri: &str, local_part: &str, arity: usize) -> Option<FUNCTION<'a>> {
        // println!("function get {:?} {:?} {:?}", uri, local_part, arity);
        if let Some(list) = self.functions.get(&self.key(uri, local_part)) {
            // println!("function list {:?}", list.len());
            //TODO: fix it!
            let rf = list.get(&arity).unwrap();
            Some(*rf)
        } else {
            // println!("function list NONE");
            None
        }
    }

    pub fn eval(&self, env: &'a mut Environment<'a>, uri: &str, local_part: &str, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
        println!("eval_builtin: {:?} {:?}", local_part, arguments);

        let fun: Option<FUNCTION> = env.functions.get(uri, local_part, arguments.len());

        if fun.is_some() {
            fun.unwrap()(env, arguments)
        } else {
            (env, Object::Empty)
        }
    }
}