use crate::values::Value;

#[derive(Debug, Clone)]
pub struct Boolean(pub(crate) bool);

impl Boolean {
    pub(crate) fn boxed(b: bool) -> Box<dyn Value> {
        Box::new(Boolean(b))
    }
}

impl Value for Boolean {

}