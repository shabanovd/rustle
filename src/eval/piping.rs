use crate::eval::{Object, object_owned_to_sequence, Environment, EvalResult, DynamicContext, Type};
use std::slice::Iter;
use crate::values::{QNameResolved, resolve_element_qname};
use crate::eval::helpers::{relax, insert_into_sequences};
use crate::eval::prolog::*;
use crate::eval::expression::Expression;
use crate::fns::object_to_bool;
use crate::parser::errors::ErrorCode;
use crate::tree::Reference;

struct SequenceIterator<'a> {
    name: &'a QNameResolved,
    iter: &'a mut Iter<'a, Object>,
    current: Option<&'a Object>,
}

impl<'a> SequenceIterator<'a> {
    fn new(name: &'a QNameResolved, iter: &'a mut Iter<'a, Object>) -> Self {
        let current = iter.next();
        SequenceIterator { name, iter, current }
    }

    fn next(&mut self) -> bool {
        self.current = self.iter.next();
        match self.current {
            Some(..) => true,
            None => false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Pipe {
    pub binding: Option<Binding>,
    pub where_expr: Option<Box<dyn Expression>>,
    pub return_expr: Option<Box<dyn Expression>>,
    pub next: Option<Box<Pipe>>,
}

pub(crate) fn eval_pipe<'a>(pipe: Box<Pipe>, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
    let mut current_env = env;

    let next = pipe.next;
    if let Some(binding) = pipe.binding {
        match binding {
            Binding::For { name, values, st, allowing_empty, positional_var } => {
                let name = resolve_element_qname(&name, &current_env);
                let positional_var = if let Some(positional_var) = positional_var {
                    Some(resolve_element_qname(&positional_var, &current_env))
                } else {
                    None
                };

                let (new_env, evaluated) = values.eval(current_env, context)?;
                current_env = new_env;

                let mut result = vec![];

                if let Some(next) = next {
                    let items = object_owned_to_sequence(evaluated);
                    if items.len() == 0 {
                        if allowing_empty {

                            let item = if let Some(st) = st.as_ref() {
                                if st.is_castable(&Object::Empty)? {
                                    Object::Empty
                                } else {
                                    return Err((ErrorCode::XPTY0004, String::from("TODO")))
                                }
                            } else {
                                Object::Empty
                            };

                            current_env.set(name.clone(), item);
                            if let Some(positional_var) = positional_var.clone() {
                                current_env.set(positional_var, Object::Atomic(Type::Integer(0)));
                            }

                            let (new_env, answer) = eval_pipe(next.clone(), current_env, context)?;
                            current_env = new_env;

                            insert_into_sequences(&mut result, answer);
                        }
                    } else {
                        let mut pos = 0;
                        for item in items {
                            pos += 1;

                            let item = if let Some(st) = st.as_ref() {
                                if st.is_castable(&item)? {
                                    item
                                } else {
                                    return Err((ErrorCode::XPTY0004, String::from("TODO")))
                                }
                            } else {
                                item
                            };

                            current_env.set(name.clone(), item);
                            if let Some(positional_var) = positional_var.clone() {
                                current_env.set(positional_var, Object::Atomic(Type::Integer(pos)));
                            }

                            let (new_env, answer) = eval_pipe(next.clone(), current_env, context)?;
                            current_env = new_env;

                            insert_into_sequences(&mut result, answer);
                        }
                    }
                }

                relax(current_env, result)
            },
            Binding::Let { name, st: type_declaration, value } => {
                let (new_env, item) = value.eval(current_env.next(), context)?;
                // let item = match item {
                //     Object::Node(rf) => {
                //         if rf.storage.is_none() {
                //             let storage = new_env.xml_tree.clone();
                //             Object::Node(
                //                 Reference { storage: Some(storage), storage_id: rf.storage_id, id: rf.id, attr_name: rf.attr_name }
                //             )
                //         } else {
                //             Object::Node(rf)
                //         }
                //     },
                //     _ => item,
                // };
                current_env = new_env.prev();

                // TODO: handle typeDeclaration

                let name = resolve_element_qname(&name, &current_env);
                current_env.set(name, item);

                if let Some(next) = next {
                    eval_pipe(next, current_env, context)
                } else {
                    Ok((current_env, Object::Empty))
                }
            },
        }
    } else if let Some(expr) = pipe.where_expr {
        let (new_env, v) = expr.eval(current_env, context)?;
        current_env = new_env;

        if object_to_bool(&v)? {
            if let Some(next) = next {
                eval_pipe(next, current_env, context)
            } else {
                Ok((current_env, Object::Empty))
            }
        } else {
            Ok((current_env, Object::Empty))
        }

    } else if let Some(expr) = pipe.return_expr {
        if let Some(..) = next {
            panic!("internal error");
        }

        expr.eval(current_env, context)
    } else {
        panic!("internal error")
    }
}