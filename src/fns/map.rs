//use crate::eval::Type;
use crate::eval::Object;
use crate::eval::Environment;

pub fn map_get<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    match arguments.as_slice() {
        [Object::Map(map), Object::Atomic(k)] => {

            println!("map_get {:?} {:?}", k, map);

            if let Some(value) = map.get(k) {
                (env, value.clone()) //TODO: understand, is it possible to avoid clone? for example by using reference all around
            } else {
                (env, Object::Empty)
            }
        }

        _ => (env, Object::Empty), // TODO: raise error?
    }
}