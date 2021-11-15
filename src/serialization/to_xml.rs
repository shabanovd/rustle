use crate::eval::{Environment, Object};
use crate::serialization::to_string::object_to_string_xml;

pub fn object_to_xml(env: &Box<Environment>, object: &Object) -> String {
    match object {
        Object::Node(rf) => {
            match rf.to_xml() {
                Ok(data) => data,
                Err(_) => panic!()
            }
        },
        _ => object_to_string_xml(env, object)
    }
}


fn fix(str: &String) -> String {
    str.replace("\"", "&quot;")
}