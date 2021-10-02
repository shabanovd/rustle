use crate::eval::Node;

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