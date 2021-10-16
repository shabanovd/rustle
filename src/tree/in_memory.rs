use std::cmp::Ordering;
use std::collections::BTreeMap;
use linked_hash_map::LinkedHashMap;
use crate::eval::expression::Expression;
use crate::tree::dln::DLN;
use crate::tree::{Reference, XMLNode, XMLTreeReader, XMLTreeWriter};
use crate::values::QName;

#[derive(Clone)]
pub struct InMemoryXMLTree {
    id: usize,

    // required during build only
    stack: Vec<DLN>,

    // state
    items: BTreeMap<DLN, Box<dyn XMLNode>>,
}

impl InMemoryXMLTree {
    pub fn create(id: usize) -> Self {
        InMemoryXMLTree {
            id,
            stack: Vec::with_capacity(42),
            items: BTreeMap::new()
        }
    }

    fn last_id(&self) -> usize {
        if let Some((k,v)) = self.items.last_key_value() {
            k.get_level_id(0)
        } else {
            1
        }
    }

    fn prepare_child(&mut self) -> DLN {
        if let Some(id) = self.stack.last() {
            let child = id.first_child();
            self.stack.push(child.clone());
            child
        } else {
            let next_id = self.last_id() + 1;
            DLN::level_id(next_id)
        }
    }

    fn next_sibling(&mut self) -> DLN {
        if let Some(id) = self.stack.pop() {
            let next = id.next_sibling();
            self.stack.push(next.clone());
            next
        } else {
            let next_id = self.last_id() + 1;
            DLN::level_id(next_id)
        }
    }
}

impl XMLTreeReader for InMemoryXMLTree {
    fn name(&self, rf: &Reference) -> Option<QName> {
        if let Some(attr_name) = &rf.attr_name {
            Some(attr_name.clone())
        } else {
            if let Some(node) = self.items.get(&rf.id) {
                node.name()
            } else {
                // TODO raise error?
                None
            }
        }
    }

    fn to_string(&self, rf: &Reference) -> Result<String, String> {
        todo!()
    }

    fn to_xml(&self, rf: &Reference) -> Result<String, String> {
        todo!()
    }

    fn typed_value_of_node(&self, rf: &Reference) -> Result<String, String> {
        let mut result = vec![];
        for (k,v) in self.items.range(&rf.id..) {
            if k == &rf.id { continue; }
            if k.start_with(&rf.id) {
                result.push(v.typed_value());
            } else {
                break;
            }
        }
        let data = result.join("");
        Ok(data)
    }

    fn cmp(&self, other: Box<&dyn XMLTreeReader>, left: &Reference, right: &Reference) -> Ordering {
        todo!()
    }
}

impl XMLTreeWriter for InMemoryXMLTree {
    fn id(&self) -> usize {
        self.id
    }

    fn as_reader(&self) -> Box<&dyn XMLTreeReader> {
        Box::new(self)
    }

    fn start_document(&mut self) -> Reference {
        let id = self.prepare_child();
        let node = Box::new(Document { id: id.clone() });

        self.items.insert(id.clone(), node);
        self.prepare_child();

        Reference { storage: None, storage_id: self.id, id, attr_name: None }
    }

    fn end_document(&mut self) -> Option<Reference> {
        match self.stack.pop() {
            Some(id) => Some(Reference { storage: None, storage_id: self.id, id, attr_name: None }),
            None => None
        }
    }

    fn start_element(&mut self, name: QName) -> Reference {
        let id = self.next_sibling();
        self.stack.push(id.clone());

        let node = Box::new(Element { id: id.clone(), name, attributes: None });

        self.items.insert(id.clone(), node);
        self.prepare_child();

        Reference { storage: None, storage_id: self.id, id, attr_name: None }
    }

    fn attribute(&mut self, name: QName, value: String) -> Reference {
        todo!()
    }

    fn end_element(&mut self) -> Option<Reference> {
        match self.stack.pop() {
            Some(id) => Some(Reference { storage: None, storage_id: self.id, id, attr_name: None }),
            None => None
        }
    }

    fn pi(&mut self, target: QName, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(PI { id: id.clone(), target, content });

        self.items.insert(id.clone(), node);

        Reference { storage: None, storage_id: self.id, id, attr_name: None }
    }

    fn text(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Text { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        Reference { storage: None, storage_id: self.id, id, attr_name: None }
    }

    fn comment(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Comment { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        Reference { storage: None, storage_id: self.id, id, attr_name: None }
    }

    fn dump(&self) -> String {
        let mut buf = String::with_capacity(100_100);
        for (id, node) in &self.items {
            buf.push_str(node.dump().as_str());
            buf.push_str("\n");
        }
        buf
    }
}

#[derive(Clone)]
struct Document {
    id: DLN,
}

impl XMLNode for Document {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn typed_value(&self) -> String {
        String::new()
    }

    fn dump(&self) -> String {
        format!("Document {{ id={} }}", self.id)
    }
}

#[derive(Clone)]
struct Element {
    id: DLN,
    name: QName,
    attributes: Option<LinkedHashMap<QName, Attribute>>,
}

impl XMLNode for Element {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        Some(self.name.clone())
    }

    fn typed_value(&self) -> String {
        String::new()
    }

    fn dump(&self) -> String {
        let attributes = if let Some(attrs) = &self.attributes {
            let mut buf = String::new();
            for (name, attr) in attrs {
                buf.push_str(" ");
                buf.push_str(format!("{:?}", name).as_str());
                buf.push_str("=");
                buf.push_str(attr.value.as_str());
            }
            buf
        } else {
            String::from("NONE")
        };
        format!("Element {{ id={}; name={:?}; attributes={}; }}",self.id, self.name, attributes)
    }
}

#[derive(Clone)]
struct Attribute {
    name: QName,
    value: String
}

impl XMLNode for Attribute {
    fn id(&self) -> DLN {
        panic!()
    }

    fn name(&self) -> Option<QName> {
        Some(self.name.clone())
    }

    fn typed_value(&self) -> String {
        self.value.clone()
    }

    fn dump(&self) -> String {
        format!("Attribute {{ name={:?}; value={}; }}", self.name, self.value)
    }
}

#[derive(Clone)]
struct PI {
    id: DLN,
    target: QName,
    content: String
}

impl XMLNode for PI {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn dump(&self) -> String {
        format!("PI {{ id={}; target={:?}; content={}; }}", self.id, self.target, self.content)
    }
}

#[derive(Clone)]
struct Text {
    id: DLN,
    content: String
}

impl XMLNode for Text {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn dump(&self) -> String {
        format!("Text {{ id={}; content={} }}", self.id, self.content)
    }
}

#[derive(Clone)]
struct Comment {
    id: DLN,
    content: String
}

impl XMLNode for Comment {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn dump(&self) -> String {
        format!("Comment {{ id={}; content={} }}", self.id, self.content)
    }
}
