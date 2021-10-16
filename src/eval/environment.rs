use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use crate::values::QNameResolved;
use crate::eval::Object;
use crate::fns::{Function, FunctionsRegister};
use crate::namespaces::*;
use crate::tree::{InMemoryXMLTree, Reference, XMLTreeWriter};

#[derive(Clone)]
pub struct Environment<'a> {
    prev: Option<Box<Environment<'a>>>,

    pub static_base_uri: Option<String>,

    pub xml_tree: Rc<Mutex<Box<dyn XMLTreeWriter>>>,

    pub namespaces: Namespaces<'a>,
    vars: HashMap<QNameResolved, Object>,
    pub functions: FunctionsRegister<'a>,

    sequence: usize,
}

impl<'a> Environment<'a> {
    pub fn create() -> Box<Self> {
        Box::new(
            Environment {
                prev: None,

                static_base_uri: None,

                xml_tree: Rc::new(Mutex::new(Box::new(InMemoryXMLTree::create(1)))),

                namespaces: Namespaces::new(),
                vars: HashMap::new(),
                functions: FunctionsRegister::new(),
                sequence: 1,
            }
        )
    }

    pub fn next(mut self) -> Box<Environment<'a>> {
        let sequence = self.next_id();
        Box::new(
            Environment {
                prev: Some(Box::new(self)),

                static_base_uri: None,

                xml_tree: Rc::new(Mutex::new(Box::new(InMemoryXMLTree::create(sequence)))),

                namespaces: Namespaces::new(),
                vars: HashMap::new(),
                functions: FunctionsRegister::new(),
                sequence: 0,
            }
        )
    }

    pub fn prev(self) -> Box<Environment<'a>> {
        match self.prev {
            Some(env) => env,
            None => panic!("internal error")
        }
    }

    pub fn xml_writer<F>(&mut self, mutation: F) -> Reference
        where F: FnOnce(&mut MutexGuard<Box<dyn XMLTreeWriter>>) -> Reference
    {
        let rf = {
            let mut w = self.xml_tree.lock().unwrap();
            mutation(&mut w)
        };
        let storage = self.xml_tree.clone();
        Reference { storage: Some(storage), storage_id: rf.storage_id, id: rf.id, attr_name: rf.attr_name }
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

    pub fn set(&mut self, name: QNameResolved, value: Object) {
        self.vars.insert(name, value);
    }

    pub fn get(&self, name: &QNameResolved) -> Option<Object> {
        self.vars.get(name).map(|val| val.clone())
    }

    pub fn declared_functions(&self, qname: &QNameResolved, arity: usize) -> Option<&Function> {
        self.functions.declared(qname, arity)
    }
}