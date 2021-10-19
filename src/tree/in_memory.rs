use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use linked_hash_map::LinkedHashMap;
use xmlparser::{ElementEnd, Token};
use crate::tree::dln::DLN;
use crate::tree::{Reference, XMLNode, XMLTreeReader, XMLTreeWriter};
use crate::values::QName;

#[derive(Clone)]
pub struct InMemoryXMLTree {
    storage: Option<Rc<Mutex<Box<dyn XMLTreeWriter>>>>,
    id: usize,

    // required during build only
    stack: Vec<DLN>,

    // state
    items: BTreeMap<DLN, Box<dyn XMLNode>>,
}

impl InMemoryXMLTree {
    fn instance(id: usize) -> Box<dyn XMLTreeWriter> {
        Box::new(InMemoryXMLTree {
            storage: None,
            id,
            stack: Vec::with_capacity(42),
            items: BTreeMap::new()
        })
    }

    pub fn create(id: usize) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let tree = InMemoryXMLTree::instance(id);

        let rf = Rc::new(Mutex::new(tree));
        let clone = rf.clone();

        {
            let mut instance = rf.lock().unwrap();
            instance.init(clone);
        }

        rf
    }

    pub fn load(id: usize, path: &str) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let data = fs::read_to_string(path).unwrap();
        InMemoryXMLTree::from_str(id, data.as_str())
    }

    pub fn from_str(id: usize, data: &str) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let mut rf = InMemoryXMLTree::create(id);
        {
            let mut tree = rf.lock().unwrap();
            tree.start_document();
            for token in xmlparser::Tokenizer::from(data) {
                match token.unwrap() {
                    Token::Declaration { version, encoding, standalone, .. } => {
                        // TODO
                    }
                    Token::ProcessingInstruction { target, content, .. } => {
                        let target = QName::local_part(target.as_str());
                        let content = if let Some(data) = content {
                            data.as_str().to_string()
                        } else {
                            String::new()
                        };
                        tree.pi(target, content);
                    }
                    Token::Comment { text, .. } => {
                        tree.comment(text.as_str().to_string());
                    },
                    Token::DtdStart { .. } => panic!(),
                    Token::EmptyDtd { .. } => panic!(),
                    Token::EntityDeclaration { .. } => panic!(),
                    Token::DtdEnd { .. } => panic!(),
                    Token::ElementStart { prefix, local, .. } => {
                        let name = QName::new(prefix.as_str().to_string(), local.as_str().to_string());
                        tree.start_element(name);
                    }
                    Token::Attribute { prefix, local, value, .. } => {
                        let name = QName::new(prefix.as_str().to_string(), local.as_str().to_string());
                        tree.attribute(name, value.as_str().to_string());
                    }
                    Token::ElementEnd { end, .. } => {
                        match end {
                            ElementEnd::Open => {}
                            ElementEnd::Close(prefix, local) => {
                                tree.end_element();
                            },
                            ElementEnd::Empty => {
                                tree.end_element();
                            }
                        }
                    }
                    Token::Text { text } => {
                        tree.text(text.as_str().to_string());
                    },
                    Token::Cdata { text, .. } => panic!()
                }
            }
            tree.end_document();
        }

        rf
    }

    pub(crate) fn as_writer(self) -> Box<dyn XMLTreeWriter> {
        Box::new(self)
    }

    fn reference(&self, id: DLN, attr_name: Option<QName>) -> Reference {
        if let Some(storage) = self.storage.clone() {
            Reference { storage, id, attr_name }
        } else {
            panic!("internal error")
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

    fn first(&self) -> Option<Reference> {
        if let Some((k, v)) = self.items.first_key_value() {
            Some(self.reference(k.clone(), None))
        } else {
            None
        }
    }

    fn children(&self, rf: &Reference) -> Result<Vec<Reference>, String> {
        let mut result = vec![];
        for (k,v) in self.items.range(&rf.id..) {
            if k == &rf.id { continue; }
            if k.start_with(&rf.id) {
                result.push(rf.clone() );
            } else {
                break;
            }
        }
        Ok(result)
    }

    fn cmp(&self, other: Box<&dyn XMLTreeReader>, left: &Reference, right: &Reference) -> Ordering {
        todo!()
    }
}

impl XMLTreeWriter for InMemoryXMLTree {
    fn init(&mut self, rf: Rc<Mutex<Box<dyn XMLTreeWriter>>>) {
        if self.storage.is_some() {
            panic!("internal error");
        }
        self.storage = Some(rf)
    }

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

        self.reference(id, None)
    }

    fn end_document(&mut self) -> Option<Reference> {
        match self.stack.pop() {
            Some(id) => Some(self.reference(id, None)),
            None => None
        }
    }

    fn start_element(&mut self, name: QName) -> Reference {
        let id = self.next_sibling();
        self.stack.push(id.clone());

        let node = Box::new(Element { id: id.clone(), name, attributes: None });

        self.items.insert(id.clone(), node);
        self.prepare_child();

        self.reference(id, None)
    }

    fn attribute(&mut self, name: QName, value: String) -> Reference {
        let size = self.stack.len();
        if size >= 2 {
            if let Some(id) = self.stack.get(size - 2) {
                if let Some(node) = self.items.get_mut(id) {
                    if node.add_attribute(name.clone(), value) {
                        return self.reference(id.clone(), Some(name));
                    } else {
                        todo!()
                    }
                }
            }
        }
        todo!()
    }

    fn end_element(&mut self) -> Option<Reference> {
        match self.stack.pop() {
            Some(id) => Some(self.reference(id, None)),
            None => None
        }
    }

    fn pi(&mut self, target: QName, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(PI { id: id.clone(), target, content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn text(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Text { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn comment(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Comment { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        if self.attributes.is_none() {
            self.attributes = Some(LinkedHashMap::new());
        }

        if let Some(attributes) = &mut self.attributes {
            attributes.insert(name.clone(), Attribute { name, value } );
        }

        true
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
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

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn dump(&self) -> String {
        format!("Comment {{ id={}; content={} }}", self.id, self.content)
    }
}
