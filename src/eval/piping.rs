use crate::eval::{Object, object_owned_to_sequence, Environment, EvalResult, eval_expr, DynamicContext};
use std::slice::Iter;
use crate::values::{QNameResolved, resolve_element_qname};
use crate::parser::op::Expr;
use crate::eval::helpers::{relax, insert_into_sequences};
use std::rc::Rc;

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

#[derive(Debug)]
pub(crate) struct Pipe {
    pub expr: Expr,
    pub next: Option<Rc<Pipe>>,
}

pub(crate) fn eval_pipe<'a>(pipe: Rc<Pipe>, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
    let mut current_env = env;

    let expr = &pipe.expr;
    let next = &pipe.next;
    match expr {
        Expr::ForBinding { name, values } => {
            let name = resolve_element_qname(&name, &current_env);
            let (new_env, evaluated) = eval_expr(*values.clone(), current_env, context)?;
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
        Expr::LetBinding { name, type_declaration,  value } => {
            let (_, item) = eval_expr(*value.clone(), current_env.clone(), context)?;

            // TODO: handle typeDeclaration

            let name = resolve_element_qname(&name, &current_env);
            current_env.set(name, item);

            if let Some(next) = next {
                eval_pipe(next.clone(), current_env, context)
            } else {
                Ok((current_env, Object::Empty))
            }
        },
        _ => {
            if let Some(next) = next {
                panic!("internal error {:?}", pipe);
            }

            eval_expr(expr.clone(), current_env, context)
        }
    }
}