use crate::eval::{Node, Object};
use crate::values::QName;

#[derive(Debug)]
pub enum XmlEvent {
    Document,
    Element { name: QName, attrs: Vec<(QName, String)>},
    Text(String),
}

impl PartialEq for XmlEvent {
    fn eq(&self, other: &Self) -> bool {
        match self {
            XmlEvent::Document => {
                match other {
                    XmlEvent::Document => true,
                    _ => false
                }
            }
            XmlEvent::Element { name: l_name, attrs: l_attrs } => {
                match other {
                    XmlEvent::Element { name: r_name, attrs: r_attrs } => {
                        if l_name != r_name && l_attrs.len() != r_attrs.len() {
                            false
                        } else {
                            for l_attr in l_attrs {
                                for r_attr in r_attrs {
                                    if l_attr.0 == r_attr.0 {
                                        if l_attr.1 !=  r_attr.1 {
                                            return false;
                                        }
                                    }
                                }
                            }
                            true
                        }
                    }
                    _ => false
                }
            }
            XmlEvent::Text(l_content) => {
                match other {
                    XmlEvent::Text(r_content) => {
                        l_content == r_content
                    },
                    _ => false
                }
            }
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub fn object_to_xml_events(obj: &Object) -> Vec<XmlEvent> {
    let mut events = vec![];
    match obj {
        Object::Node(node) => {
            private_node_to_events(node, &mut events);
        }
        _ => {}
    }
    events
}

// TODO refactor into iterator
fn private_node_to_events(node: &Node, events: &mut Vec<XmlEvent>) {
    match node {
        Node::Document { children, .. } => {
            events.push(XmlEvent::Document);
            for child in children {
                private_node_to_events(child, events);
            }
        }
        Node::Element { name, attributes, children, .. } => {
            let mut attrs = Vec::with_capacity(attributes.len());
            for attribute in attributes {
                match attribute {
                    Node::Attribute { name, value, .. } => {
                        attrs.push((name.clone(), value.clone()));
                    },
                    _ => {} // TODO: error?
                }
            }
            events.push(XmlEvent::Element { name: name.clone(), attrs });

            for child in children {
                private_node_to_events(child, events);
            }
        }
        Node::Attribute { .. } => panic!(),
        Node::Text { content, .. } => {
            let mut adding = true;
            if events.len() > 0 {
                let last = events.remove(events.len() - 1);
                match last {
                    XmlEvent::Text(last_content) => {
                        let mut new_content = last_content.clone();
                        new_content.push_str(content.as_str());

                        events.push(XmlEvent::Text(new_content));

                        adding = false;
                    }
                    _ => {
                        events.push(last);
                    }
                }
            }
            if adding {
                events.push(XmlEvent::Text(content.clone()));
            }
        }
        Node::Comment { .. } => panic!(),
        Node::PI { .. } => panic!(),
    }
}

pub fn node_to_string(node: &Node) -> String {
    match node {
        Node::Document { children, .. } => {
            let mut buf = String::new();
            for child in children {
                buf.push_str(node_to_string(child).as_str());
            }
            buf
        }
        Node::Element { name, attributes, children, .. } => {
            let mut buf = String::new();
            buf.push_str("<");
            buf.push_str(name.string().as_str());

            for attribute in attributes {
                match attribute {
                    Node::Attribute { name, value, .. } => {
                        buf.push_str(" ");
                        buf.push_str(name.string().as_str());
                        buf.push_str("=\"");
                        buf.push_str(fix(value).as_str());
                        buf.push_str("\"");
                    },
                    _ => panic!("error: {:?}", attribute)
                }
            }

            if children.len() == 0 {
                buf.push_str("/>");
            } else {
                buf.push_str(">");

                for child in children {
                    buf.push_str(node_to_string(child).as_str());
                }

                buf.push_str("</");
                buf.push_str(name.string().as_str());
                buf.push_str(">")
            }
            buf
        }
        Node::Attribute { .. } => { todo!() }
        Node::Text { content, .. } => { content.clone() }
        Node::Comment { content, .. } => {
            let mut buf = String::new();

            buf.push_str("<!--");
            buf.push_str(content.as_str());
            buf.push_str("-->");

            buf
        }
        Node::PI { target, content, .. } => {
            let mut buf = String::new();

            buf.push_str("<?");
            buf.push_str(target.string().as_str());
            buf.push_str(" ");
            buf.push_str(content.as_str());
            buf.push_str("?>");

            buf
        }
    }
}

fn fix(str: &String) -> String {
    str.replace("\"", "&quot;")
}