use crate::eval::{Object, object_owned_to_sequence, Environment, EvalResult, DynamicContext};
use std::slice::Iter;
use crate::values::{QNameResolved, resolve_element_qname};
use crate::eval::helpers::{relax, insert_into_sequences};
use crate::eval::prolog::*;
use crate::eval::expression::Expression;

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
    pub expr: Option<Box<dyn Expression>>,
    pub next: Option<Box<Pipe>>,
}

pub(crate) fn eval_pipe<'a>(pipe: Box<Pipe>, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
    let mut current_env = env;

    let next = pipe.next;
    if let Some(binding) = pipe.binding {
        match binding {
            Binding::For { name, values } => {
                let name = resolve_element_qname(&name, &current_env);
                let (new_env, evaluated) = values.eval(current_env, context)?;
                current_env = new_env;

                let mut result = vec![];

                if let Some(next) = next {
                    let items = object_owned_to_sequence(evaluated);
                    for item in items {
                        current_env.set(name.clone(), item);

                        let (new_env, answer) = eval_pipe(next.clone(), current_env, context)?;
                        current_env = new_env;

                        insert_into_sequences(&mut result, answer);
                    }
                }

                relax(current_env, result)
            },
            Binding::Let { name, type_declaration, value } => {
                let (_, item) = value.eval(current_env.clone(), context)?;

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
    } else if let Some(expr) = pipe.expr {
        if let Some(..) = next {
            panic!("internal error");
        }

        expr.eval(current_env, context)
    } else {
        panic!("internal error")
    }
}