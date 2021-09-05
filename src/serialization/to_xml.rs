use crate::eval::Node;

pub fn node_to_string(node: &Node) -> String {
    match node {
        Node::Node { name, attributes, children, .. } => {
            let mut result = String::new();
            result.push_str("<");
            result.push_str(name.string().as_str());

            for attribute in attributes {
                match attribute {
                    Node::Attribute { name, value, .. } => {
                        result.push_str(" ");
                        result.push_str(name.string().as_str());
                        result.push_str("=\"");
                        result.push_str(fix(value).as_str());
                        result.push_str("\"");
                    },
                    _ => panic!("error: {:?}", attribute)
                }
            }

            if children.len() == 0 {
                result.push_str("/>");
            } else {
                result.push_str(">");

                for child in children {
                    result.push_str(node_to_string(child).as_str());
                }

                result.push_str("</");
                result.push_str(name.string().as_str());
                result.push_str(">")
            }
            result
        }
        Node::Attribute { .. } => { todo!() }
        Node::NodeText { content, .. } => { content.clone() }
        Node::NodeComment { content, .. } => {
            let mut result = String::new();

            result.push_str("<!--");
            result.push_str(content.as_str());
            result.push_str("-->");

            result
        }
        Node::NodePI { target, content, .. } => {
            let mut result = String::new();

            result.push_str("<?");
            result.push_str(target.string().as_str());
            result.push_str(" ");
            result.push_str(content.as_str());
            result.push_str("?>");

            result
        }
    }
}

fn fix(str: &String) -> String {
    str.replace("\"", "&quot;")

}