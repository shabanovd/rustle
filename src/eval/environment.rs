use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use crate::values::{QName, QNameResolved};
use crate::eval::Object;
use crate::eval::prolog::{BoundarySpace, ConstructionMode, DecimalFormatPropertyName, EmptyOrderMode, InheritMode, OrderingMode, PreserveMode};
use crate::fns::{Function, FUNCTION, FunctionsRegister};
use crate::namespaces::*;
use crate::tree::{InMemoryXMLTree, Reference, XMLTreeWriter};

#[derive(Clone)]
pub struct Environment {
    prev: Option<Box<Environment>>,

    pub boundary_space: Option<BoundarySpace>,
    pub default_collation: Option<String>,
    pub static_base_uri: Option<String>,
    pub construction_mode: Option<ConstructionMode>,
    pub ordering_mode: Option<OrderingMode>,
    pub empty_order_mode: Option<EmptyOrderMode>,
    pub copy_namespaces: Option<(PreserveMode, InheritMode)>,
    pub decimal_formats: Option<HashMap<Option<QName>, HashMap<DecimalFormatPropertyName, String>>>,

    pub xml_tree: Rc<Mutex<Box<dyn XMLTreeWriter>>>,

    pub namespaces: Namespaces,
    vars: HashMap<QNameResolved, Object>,
    pub functions: FunctionsRegister,

    sequence: usize,
}

impl Environment {
    pub fn create() -> Box<Self> {
        Box::new(
            Environment {
                prev: None,

                boundary_space: None,
                default_collation: None,
                static_base_uri: None,
                construction_mode: None,
                ordering_mode: None,
                empty_order_mode: None,
                copy_namespaces: None,
                decimal_formats: None,

                xml_tree: InMemoryXMLTree::create(1),

                namespaces: Namespaces::new(),
                vars: HashMap::new(),
                functions: FunctionsRegister::new(),
                sequence: 1,
            }
        )
    }

    pub fn next(mut self) -> Box<Environment> {
        let sequence = self.next_id();
        Box::new(
            Environment {
                prev: Some(Box::new(self)),

                boundary_space: None,
                default_collation: None,
                static_base_uri: None,
                construction_mode: None,
                ordering_mode: None,
                empty_order_mode: None,
                copy_namespaces: None,
                decimal_formats: None,

                xml_tree: InMemoryXMLTree::create(sequence),

                namespaces: Namespaces::new(),
                vars: HashMap::new(),
                functions: FunctionsRegister::new(),
                sequence: 0,
            }
        )
    }

    pub fn set_option(&self, name: QName, value: String) {
        // TODO
    }

    pub fn prev(self) -> Box<Environment> {
        match self.prev {
            Some(env) => env,
            None => panic!("internal error")
        }
    }

    fn unwind<T, F: FnMut(&Environment) -> Option<T>>(&self, mut op: F) -> Option<T> {
        let mut env = self;
        loop {
            if let Some(obj) = op(env) {
                break Some(obj);
            } else if let Some(prev) = &env.prev {
                env = prev
            } else {
                break None
            }
        }
    }

    pub fn xml_writer<F>(&mut self, mutation: F) -> Reference
        where F: FnOnce(&mut MutexGuard<Box<dyn XMLTreeWriter>>) -> Reference
    {
        let mut w = self.xml_tree.lock().unwrap();
        mutation(&mut w)
    }

    pub fn xml_tree_id(&self) -> usize {
        self.xml_tree.lock().unwrap().id()
    }

    pub fn next_id(&mut self) -> usize {
        match &mut self.prev {
            Some(prev) => prev.next_id(),
            None => {
                self.sequence += 1;
                self.sequence
            }
        }
    }

    pub fn default_namespace_for_element(&self) -> String {
        match self.unwind(|env| env.namespaces.default_for_element.clone()) {
            Some(ns) => ns,
            None => "".to_string()
        }
    }

    pub fn default_namespace_for_function(&self) -> String {
        match self.unwind(|env| env.namespaces.default_for_function.clone()) {
            Some(ns) => ns,
            None => XPATH_FUNCTIONS.uri.to_string()
        }
    }

    pub fn set_variable(&mut self, name: QNameResolved, value: Object) {
        // println!("set_variable {:?} {:?}", name, value);
        self.vars.insert(name, value);
    }

    pub fn get_variable(&self, name: &QNameResolved) -> Option<Object> {
        self.unwind(|env| env.vars.get(name).map(|val| val.clone()))
    }

    pub fn get_function(&self, name: &QNameResolved, arity: usize) -> Option<FUNCTION> {
        self.unwind(|env| env.functions.get(name, arity).map(|val| val.clone()))
    }

    pub fn declared_functions(&self, name: &QNameResolved, arity: usize) -> Option<&Function> {
        // TODO self.unwind(|env| env.functions.declared(name, arity))
        let obj = self.functions.declared(name, arity);
        if obj.is_some() {
            obj
        } else if let Some(prev) = &self.prev {
            prev.declared_functions(name, arity)
        } else {
            None
        }
    }
}