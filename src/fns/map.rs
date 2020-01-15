use crate::eval::Type;
use crate::eval::Object;
use crate::eval::Environment;

pub fn map_get<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    match arguments.as_slice() {
        [Object::Map(map), Object::Atomic(key)] => {

            let value = map.get(key).unwrap();

            (env, value.clone()) //TODO: understand, is it possible to avoid clone? for example by using reference all around
        }

        _ => (env, Object::Empty), // TODO: raise error
    }
}