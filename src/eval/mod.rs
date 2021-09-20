use std::collections::HashMap;

use crate::eval::Object::Empty;
use crate::fns::call;
use crate::parser::op::{Statement, Expr, ItemType, OccurrenceIndicator, OperatorComparison};
use crate::values::{resolve_element_qname, resolve_function_qname};

pub use self::environment::Environment;
use crate::fns::object_to_bool;
use crate::serialization::object_to_string;
use crate::serialization::to_string::object_to_string_xml;
use crate::parser::errors::ErrorCode;

mod environment;
pub(crate) mod comparison;
mod value;
pub(crate) use value::*;

use crate::values::*;

mod arithmetic;
use arithmetic::eval_arithmetic;
use crate::eval::comparison::eval_comparison;
use crate::eval::arithmetic::eval_unary;

pub(crate) mod helpers;
use helpers::*;

pub type EvalResult<'a> = Result<(Box<Environment<'a>>, Object), (ErrorCode, String)>;

const DEBUG: bool = false;

pub fn eval_statements(statements: Vec<Statement>, env: Box<Environment>) -> Result<Object, (ErrorCode, String)> {

    let mut result = Object::Empty;

    let mut current_env = env;

    for statement in statements {
        let (new_env, new_result) = eval_statement(statement, current_env)?;
        current_env = new_env;

        result = new_result;

        if let &Object::Return(_) = &result {
            return Ok(result);
        }
    }

    if DEBUG {
        println!("result: {:?}", result);
    }

    Ok(result)
}

fn eval_statement(statement: Statement, env: Box<Environment>) -> EvalResult {
    match statement {
        Statement::Prolog(exprs) => Ok((eval_prolog(exprs, env), Object::Nothing)),
        Statement::Program(expr) => eval_expr(expr, env, &Object::Nothing),
    }
}

pub fn eval_prolog(exprs: Vec<Expr>, env: Box<Environment>) -> Box<Environment> {
    let mut current_env = env;

    for expr in exprs {
        current_env = eval_prolog_expr(expr, current_env);
    }

    current_env
}

fn eval_prolog_expr(expression: Expr, env: Box<Environment>) -> Box<Environment> {
    if DEBUG {
        println!("eval_expr: {:?}", expression);
    }

    let mut current_env = env;

    match expression {
        Expr::AnnotatedDecl { annotations, decl } => {
            // TODO handle annotations

            eval_prolog_expr(*decl, current_env)
        },
        Expr::FunctionDecl { name, params, type_declaration, external, body } => {
            let name = resolve_function_qname(name, &current_env);

            // TODO: handle typeDeclaration

            if let Some(body) = body {
                current_env.functions.put(name, params, body);

            } else {
                panic!("error")
            }

            current_env
        },
        Expr::VarDecl { name, type_declaration, external, value } => {
            let name = resolve_element_qname(name, &current_env);

            if let Some(expr) = *value {
                println!("expr {:?}", expr);
                match eval_expr(expr, current_env.clone(), &Object::Nothing) {
                    Ok((new_env, obj)) => {
                        current_env.set(name, obj);
                    },
                    Err((code, msg)) => panic!("Error {:?} {:?}", code, msg),
                }
            }

            current_env
        },
        _ => panic!("unexcpected at prolog {:?}", expression)
    }
}

pub fn eval_exprs<'a>(exprs: Vec<Expr>, env: Box<Environment<'a>>, context_item: &Object) -> EvalResult<'a> {

    let mut result = Object::Empty;

    let mut current_env = env;

    for expr in exprs {
        let (new_env, new_result) = eval_expr(expr, current_env, context_item)?;
        current_env = new_env;

        // TODO: review it
        result = new_result;

        if let &Object::Return(_) = &result {
            return Ok((current_env, result));
        }
    }

    Ok((current_env, result))
}

pub fn eval_expr<'a>(expression: Expr, env: Box<Environment<'a>>, context_item: &Object) -> EvalResult<'a> {
    if DEBUG {
        println!("eval_expr: {:?}", expression);
    }

    let mut current_env = env;

    match expression {
        Expr::Boolean(bool) =>
            Ok((current_env, Object::Atomic(Type::Boolean(bool)))),
        Expr::Integer(number) =>
            Ok((current_env, Object::Atomic(Type::Integer(number)))),
        Expr::Decimal(number) =>
            Ok((current_env, Object::Atomic(Type::Decimal(number)))),
        Expr::Double(number) =>
            Ok((current_env, Object::Atomic(Type::Double(number)))),
        Expr::String(string) =>
            Ok((current_env, Object::Atomic(Type::String(string)))),
        Expr::StringComplex(exprs) => {
            let mut strings = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let (new_env, object) = eval_expr(expr, current_env, context_item)?;
                current_env = new_env;

                let str = object_to_string(&object);
                strings.push(str);
            }

            Ok((current_env, Object::Atomic(Type::String(strings.join("")))))
        },
        Expr::EscapeQuot => Ok((current_env, Object::Atomic(Type::String(String::from("\""))))),
        Expr::EscapeApos => Ok((current_env, Object::Atomic(Type::String(String::from("'"))))),
        Expr::CharRef { representation, reference } => {
            Ok((current_env, Object::CharRef { representation, reference }))
        },
        Expr::EntityRef(reference) => {
            Ok((current_env, Object::EntityRef(reference)))
        },

        Expr::ContextItem => {
            Ok((current_env, context_item.clone()))
        },

        Expr::QName { local_part, url, prefix } => {
            Ok((current_env, Object::Atomic( Type::QName { local_part, url, prefix } ) ))
        },

        Expr::Body(exprs) => {
            if exprs.len() == 0 {
                Ok((current_env, Object::Empty))
            } else if exprs.len() == 1 {
                let expr = exprs.get(0).unwrap().clone();

                let (new_env, value) = eval_expr(expr, current_env, context_item)?;
                current_env = new_env;

                Ok((current_env, value))
            } else {
                let mut evaluated = vec![];
                for expr in exprs {
                    let (new_env, value) = eval_expr(expr, current_env, context_item)?;
                    current_env = new_env;

                    match value {
                        Object::Empty => {},
                        _ => evaluated.push(value)
                    }
                }

                if evaluated.len() == 0 {
                    Ok((current_env, Object::Empty))
                } else if evaluated.len() == 1 {
                    Ok((current_env, evaluated[0].clone()))
                } else {
                    // TODO understand when it should happen... sort_and_dedup(&mut evaluated);
                    Ok((current_env, Object::Sequence(evaluated)))
                }
            }
        },

        Expr::Steps(steps) => {
            let mut current_context_item = context_item.clone();
            for step in steps {
                println!("step {:?}", step);

                let (new_env, value) = eval_expr(step, current_env, &current_context_item)?;
                current_env = new_env;

                current_context_item = value;
                println!("result {:?}", current_context_item);
            }

            Ok((current_env, current_context_item))
        },
        Expr::Path { steps,  expr } => {
            eval_expr(*expr, current_env, context_item)
        }

        Expr::AxisStep { step, predicates } => {
            let (new_env, value) = eval_expr(*step, current_env, context_item)?;
            current_env = new_env;

            eval_predicates(predicates, current_env, value, context_item)
        },
        Expr::ForwardStep { attribute, test} => {
            println!("context_item {:?}", context_item);
            if attribute {
                step_and_test(Axis::ForwardAttribute, *test, current_env, context_item)
            } else {
                step_and_test(Axis::ForwardChild, *test, current_env, context_item)
            }
        },

        Expr::NodeElement { name, attributes , children } => {
            let (new_env, evaluated_name) = eval_expr(*name, current_env, context_item)?;
            current_env = new_env;

            let evaluated_name = object_to_qname(evaluated_name);
            let mut evaluated_attributes = vec![];
            for attribute in attributes {
                let (new_env, evaluated_attribute) = eval_expr(attribute, current_env, context_item)?;
                current_env = new_env;

                match evaluated_attribute {
                    Object::Node(Node::Attribute { sequence, name, value}) => { // TODO: avoid copy!
                        let evaluated_attribute = Node::Attribute { sequence, name, value };
                        evaluated_attributes.push(evaluated_attribute);
                    }
                    _ => panic!("unexpected object") //TODO: better error
                };
            }

            let mut evaluated_children = vec![];
            for child in children {
                let (new_env, evaluated_child) = eval_expr(child, current_env, context_item)?;
                current_env = new_env;

                match evaluated_child {
                    Object::Sequence(items) => {
                        let mut add_space = false;
                        for item in items {
                            let id = current_env.next_id();
                            match item {
                                Object::Node(Node::Attribute { sequence, name, value}) => { // TODO: avoid copy!
                                    add_space = false;

                                    let evaluated_attribute = Node::Attribute { sequence, name, value };

                                    evaluated_attributes.push(evaluated_attribute);
                                },
                                Object::Node(node) => {
                                    add_space = false;

                                    evaluated_children.push(node);
                                }
                                Object::Atomic(..) => {
                                    let mut content = object_to_string_xml(&item);
                                    if add_space {
                                        content.insert(0, ' ');
                                    }
                                    evaluated_children.push(Node::NodeText { sequence: -1, content });

                                    add_space = true;
                                }
                                _ => panic!("unexpected object {:?}", item) //TODO: better error
                            }
                        }
                    },
                    Object::Node(Node::Attribute { sequence, name, value}) => { // TODO: avoid copy!
                        let evaluated_attribute = Node::Attribute { sequence, name, value };

                        evaluated_attributes.push(evaluated_attribute);
                    },
                    Object::Node(node) => {
                        evaluated_children.push(node);
                    },
                    Object::Atomic(..) => {
                        let content = object_to_string(&evaluated_child);
                        evaluated_children.push(Node::NodeText { sequence: -1, content });
                    }
                    _ => panic!("unexpected object {:?}", evaluated_child) //TODO: better error
                };
            }

            let id = current_env.next_id();
            Ok((current_env, Object::Node(
                Node::Node { sequence: id, name: evaluated_name, attributes: evaluated_attributes, children: evaluated_children }
            )))
        },

        Expr::NodeAttribute { name, value } => {
            let (new_env, evaluated_name) = eval_expr(*name, current_env, context_item)?;
            current_env = new_env;

            let evaluated_name = object_to_qname(evaluated_name);

            let (new_env, evaluated_value) = eval_expr(*value, current_env, context_item)?;
            current_env = new_env;

            let evaluated_value = match evaluated_value {
                Object::Atomic(Type::String(string)) => { // TODO: avoid copy!
                    string
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            let id = current_env.next_id();

            Ok((current_env, Object::Node(
                Node::Attribute { sequence: id, name: evaluated_name, value: evaluated_value }
            )))
        },

        Expr::NodeText(content) => {
            let (new_env, evaluated) = eval_expr(*content, current_env, context_item)?;
            current_env = new_env;

            let content = object_to_string(&evaluated);

            let id = current_env.next_id();
            Ok((current_env, Object::Node(Node::NodeText { sequence: id, content })))
        },
        Expr::NodeComment(content) => {
            let (new_env, evaluated) = eval_expr(*content, current_env, context_item)?;
            current_env = new_env;

            let content = object_to_string(&evaluated);

            let id = current_env.next_id();
            Ok((current_env, Object::Node(Node::NodeComment { sequence: id, content })))
        },
        Expr::NodePI { target, content } => {
            let (new_env, evaluated_target) = eval_expr(*target, current_env, context_item)?;
            current_env = new_env;

            let target = object_to_qname(evaluated_target);

            let (new_env, evaluated) = eval_expr(*content, current_env, context_item)?;
            current_env = new_env;

            let content = object_to_string(&evaluated);

            let id = current_env.next_id();
            Ok((current_env, Object::Node(Node::NodePI { sequence: id, target, content })))
        },

        Expr::Map { entries } => {
            let mut map = HashMap::new();
            for entry in entries {
                match entry {
                    Expr::MapEntry { key, value } => {
                        let (new_env, evaluated_key) = eval_expr(*key, current_env, context_item)?;
                        current_env = new_env;

                        let (new_env, evaluated_value) = eval_expr(*value, current_env, context_item)?;
                        current_env = new_env;

                        match evaluated_key {
                            Object::Atomic(key_object) => {
                                map.insert(key_object, evaluated_value);
                            }
                            _ => panic!("wrong expression") //TODO: proper code
                        }
                    }
                    _ => panic!("wrong expression") //TODO: proper code
                }
            }

            Ok((current_env, Object::Map(map)))
        },

        Expr::SimpleMap(exprs)  => {
            let mut result = Object::Empty;
            let mut i = 0;
            for expr in exprs {
                if i == 0 {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item)?;
                    current_env = new_env;

                    result = evaluated;
                } else {
                    let mut sequence = vec![];

                    let it = object_to_iterator(&result);
                    for item in it {
                        let (new_env, evaluated) = eval_expr(expr.clone(), current_env, &item)?;
                        current_env = new_env;

                        let items = object_owned_to_sequence(evaluated);
                        relax_sequences(&mut sequence, items);
                    }
                    sort_and_dedup(&mut sequence);
                    result = Object::Sequence(sequence);
                }
                i += 1;
            }
            Ok((current_env, result))
        },

        Expr::Unary { expr, sign_is_positive } => {
            let (new_env, evaluated) = eval_expr(*expr, current_env, context_item)?;
            current_env = new_env;

            process_items(current_env, evaluated, |env, item| {
                match item {
                    Object::Empty => Ok((env, Object::Empty)),
                    _ => eval_unary(env, item, sign_is_positive)
                }
            })
        },
        Expr::Binary { left, operator, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env, context_item)?;
            current_env = new_env;

            if left_result == Object::Empty {
                Ok((current_env, Object::Empty))
            } else {
                let (new_env, right_result) = eval_expr(*right, current_env, context_item)?;
                current_env = new_env;

                eval_arithmetic(current_env, operator, left_result, right_result)
            }
        },
        Expr::Comparison { left, operator, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env, context_item)?;
            current_env = new_env;

            let (new_env, right_result) = eval_expr(*right, current_env, context_item)?;
            current_env = new_env;

            eval_comparison(current_env, operator, left_result, right_result)
        },

        Expr::Call {function, arguments} => {
            let name = resolve_function_qname(function, &current_env);

            let (parameters, body) = match current_env.get(&name ) {
                Some(Object::Function {parameters, body}) => (parameters, body),
                None => {
                    let mut evaluated_arguments = vec![];
                    for argument in arguments {
                        let (new_env, value) = eval_expr(argument, current_env, context_item)?;
                        current_env = new_env;

                        evaluated_arguments.push(value);
                    }

                    return call(current_env, name, evaluated_arguments, context_item);
                }
                _ => panic!("error")
            };

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {
                let (new_env, new_result) = eval_expr(argument, current_env, context_item)?;
                current_env = new_env;

                let name = resolve_function_qname(parameter.name, &current_env);

                function_environment.set(name, new_result);
            }

            let (_, result) = eval_expr(*body, Box::new(function_environment), context_item)?;

            Ok((current_env, result))
        },

        Expr::Range { from, till } => {
            let (new_env, evaluated_from) = eval_expr(*from, current_env, context_item)?;
            current_env = new_env;

            let (new_env, evaluated_till) = eval_expr(*till, current_env, context_item)?;
            current_env = new_env;

            let min = match evaluated_from {
                Object::Atomic(t) => type_to_int(t),
                _ => panic!("from is not atomic")
            };

            let max = match evaluated_till {
                Object::Atomic(t) => type_to_int(t),
                _ => panic!("till is not atomic")
            };

            if min > max {
                Ok((current_env, Object::Empty))
            } else if min == max {
                Ok((current_env, Object::Atomic(Type::Integer(min))))
            } else {
                Ok((current_env, Object::Range { min, max }))
            }
        },

        Expr::SquareArrayConstructor(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                let (new_env, evaluated) = eval_expr(item, current_env, context_item)?;
                current_env = new_env;

                values.push(evaluated);
            }

            Ok((current_env, Object::Array(values)))
        },

        Expr::CurlyArrayConstructor(expr) => {
            let (new_env, evaluated) = eval_expr(*expr, current_env, context_item)?;
            current_env = new_env;

            let values = match evaluated {
                Object::Empty => vec![],
                _ => panic!("can't convert to array {:?}", evaluated)
            };

            Ok((current_env, Object::Array(values)))
        },

        Expr::Postfix { primary, suffix } => {
            let (new_env, value) = eval_expr(*primary, current_env, context_item)?;
            current_env = new_env;

            eval_predicates(suffix, current_env, value, context_item)
        },

        Expr::SequenceEmpty() => {
            Ok((current_env, Object::Empty))
        },
        Expr::Sequence(expr) => {
            let (new_env, value) = eval_expr(*expr, current_env, context_item)?;
            current_env = new_env;

            let mut items = object_owned_to_sequence(value);
            let mut result= Vec::with_capacity(items.len());
            relax_sequences(&mut result, items);
            relax(current_env, result)
        },

        Expr::Or(exprs) => {
            if exprs.len() == 0 {
                Ok((current_env, Object::Empty))
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item)?;
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                if sequence.len() == 0 {
                    Ok((current_env, Object::Empty))
                } else if sequence.len() == 1 {
                    let object = sequence.remove(0);
                    Ok((current_env, object))
                } else {
                    let result = sequence.into_iter()
                        .map(|item| object_to_bool(&item))
                        .fold(true, |acc, value| acc || value );

                    Ok((current_env, Object::Atomic(Type::Boolean(result))))
                }
            }
        },
        Expr::And(exprs) => {
            let result = if exprs.len() == 0 {
                Object::Empty
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item)?;
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                let result: Object = if sequence.len() == 0 {
                    Object::Empty
                } else if sequence.len() == 1 {
                    sequence.remove(0)
                } else {
                    let result = sequence.into_iter()
                        .map(|item| object_to_bool(&item))
                        .fold(true, |acc, value| acc && value );

                    Object::Atomic(Type::Boolean(result))
                };
                result
            };
            Ok((current_env, result))
        },
        Expr::StringConcat(exprs) => {
            if exprs.len() == 0 {
                Ok((current_env, Object::Atomic(Type::String(String::new()))))
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item)?;
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                if sequence.len() == 0 {
                    Ok((current_env, Object::Atomic(Type::String(String::new()))))
                } else if sequence.len() == 1 {
                    let object = sequence.remove(0);
                    Ok((current_env, object))
                } else {
                    let str = sequence.into_iter()
                        .map(|item| object_to_string(&item))
                        .collect();

                    Ok((current_env, Object::Atomic(Type::String(str))))
                }
            }
        },

        Expr::Union(exprs) => {
            let mut result = vec![];
            for expr in exprs {
                let (new_env, items) = eval_expr(expr, current_env, context_item)?;
                current_env = new_env;

                let mut items = object_owned_to_sequence(items);

                join_sequences(&mut result, items);
                sort_and_dedup(&mut result)
            }

            relax(current_env, result)
        },

        Expr::NamedFunctionRef { name, arity } => {
            let (new_env, arity) = eval_expr(*arity, current_env, context_item)?;
            current_env = new_env;

            let arity = object_to_integer(arity);
            // TODO: check arity value
            let arity = arity as usize;

            let name = resolve_function_qname(name, &current_env);

            Ok((current_env, Object::FunctionRef { name, arity }))
        },

        Expr::Function { arguments, body } => {
            Ok((current_env, Object::Function { parameters: arguments, body }))
        },

        Expr::If { condition, consequence, alternative } => {
            let (new_env, evaluated) = eval_expr(*condition, current_env, context_item)?;
            current_env = new_env;

            if object_to_bool(&evaluated) {
                let (new_env, evaluated) = eval_expr(*consequence, current_env, context_item)?;
                current_env = new_env;

                Ok((current_env, evaluated))
            } else {
                let (new_env, evaluated) = eval_expr(*alternative, current_env, context_item)?;
                current_env = new_env;

                Ok((current_env, evaluated))
            }
        },

        Expr::FLWOR { clauses, return_expr } => {
            // TODO: new env?
            // TODO: handle  WhereClause | GroupByClause | OrderByClause | CountClause
            let (new_env, _) = eval_exprs(clauses, current_env, context_item)?;
            current_env = new_env;

            // println!("returnExpr {:#?}", returnExpr);

            let (new_env, evaluated) = eval_expr(*return_expr, current_env, context_item)?;
            current_env = new_env;

            Ok((current_env, evaluated))
        },
        Expr::LetClause { bindings } => {
            for binding in bindings {
                let (new_env, _) = eval_expr(binding, current_env, context_item)?;
                current_env = new_env;
            }

            Ok((current_env, Object::Empty))
        },
        Expr::LetBinding { name, type_declaration,  value } => {
            let (_, evaluated_value) = eval_expr(*value, current_env.clone(), context_item)?;

            // TODO: handle typeDeclaration

            let name = resolve_element_qname(name, &current_env);

            let mut new_env = *current_env.clone();
            new_env.set(name, evaluated_value);

            Ok((Box::new(new_env), Object::Empty))
        },
        Expr::ForClause { bindings } => {
            for binding in bindings {
                match binding {
                    Expr::ForBinding { name, values } => {
                        let (new_env, evaluated) = eval_expr(*values, current_env, context_item)?;
                        current_env = new_env;

                        let name = resolve_element_qname(name, &current_env);
                        current_env.set(name.clone(), Object::ForBinding { name, values: Box::new(evaluated) } );
                    },
                    _ => panic!("internal error")
                }
            }

            Ok((current_env, Object::Empty))
        },
        Expr::VarRef { name } => {

            let name = resolve_element_qname(name, &current_env);

            if let Some(value) = current_env.get(&name) {
                Ok((current_env, value))
            } else {
                panic!("unknown variable {:?}", name)
            }
        },

        Expr::Treat { expr, st } => {
            let (new_env, object) = eval_expr(*expr, current_env, context_item)?;
            current_env = new_env;

            let (item_type, occurrence_indicator) = match *st {
                Expr::SequenceType { item_type, occurrence_indicator } => {
                    (item_type, occurrence_indicator)
                },
                _ => panic!("unexpected {:?}", st)
            };

            // TODO occurrence_indicator checks

            let result = match item_type {
                ItemType::AtomicOrUnionType(name) => {
                    match object {
                        Object::Empty => {
                            occurrence_indicator == OccurrenceIndicator::ZeroOrMore
                            || occurrence_indicator == OccurrenceIndicator::ZeroOrOne
                        },
                        Object::Atomic(Type::String(..)) => name == *XS_STRING,
                        Object::Atomic(Type::NormalizedString(..)) => name == *XS_STRING,
                        Object::Atomic(Type::Integer(..)) => name == *XS_INTEGER,
                        Object::Atomic(Type::Decimal{..}) => name == *XS_DECIMAL,
                        Object::Atomic(Type::Float{..}) => name == *XS_FLOAT,
                        Object::Atomic(Type::Double{..}) => name == *XS_DOUBLE,
                        _ => panic!("TODO: {:?}", object)
                    }
                },
                _ => panic!("TODO: {:?}", item_type)
            };

            Ok((current_env, Object::Atomic(Type::Boolean(result))))
        },

        Expr::Castable { expr, st } => {
            let (new_env, object) = eval_expr(*expr, current_env, context_item)?;
            current_env = new_env;

            println!("st {:?}", st);

            Ok((current_env, object))
        },

        _ => panic!("TODO {:?}", expression)
    }
}

#[allow(dead_code)]
enum Axis {
    ForwardChild,
    ForwardDescendant,
    ForwardAttribute,
    ForwardSelf,
    ForwardDescendantOrSelf,
    ForwardFollowingSibling,
    ForwardFollowing,

    ReverseParent,
    ReverseAncestor,
    ReversePrecedingSibling,
    ReversePreceding,
    ReverseAncestorOrSelf,
}

fn step_and_test<'a>(step: Axis, test: Expr, env: Box<Environment<'a>>, context_item: &Object) -> EvalResult<'a> {
    match context_item {
        Object::Nothing => {
            panic!("XPDY0002")
        },
        Object::Node(node) => {
            match node {
                Node::Node { sequence, name, attributes, children } => {
                    match step {
                        Axis::ForwardChild => {
                            let mut result = vec![];
                            for child in children {
                                if test_node(&test, child) {
                                    result.push(Object::Node(child.clone()))
                                }
                            }

                            if result.len() == 0 {
                                Ok((env, Object::Empty))
                            } else if result.len() == 1 {
                                Ok((env, result.remove(0)))
                            } else {
                                Ok((env, Object::Sequence(result)))
                            }
                        },
                        Axis::ForwardAttribute => {
                            let mut result = vec![];
                            for attribute in attributes {
                                if test_node(&test, attribute) {
                                    result.push(Object::Node(attribute.clone()))
                                }
                            }

                            if result.len() == 0 {
                                Ok((env, Object::Empty))
                            } else if result.len() == 1 {
                                Ok((env, result.remove(0)))
                            } else {
                                Ok((env, Object::Sequence(result)))
                            }
                        }
                        _ => todo!()
                    }
                },
                _ => Ok((env, Object::Empty))
            }
        },
        _ => Ok((env, Object::Empty))
    }
}

fn test_node(test: &Expr, node: &Node) -> bool {
    match test {
        Expr::NameTest(qname) => {
            match node {
                Node::Node { sequence, name, attributes, children } => {
                    qname.local_part == name.local_part && qname.url == name.url
                },
                Node::Attribute { sequence, name, value } => {
                    qname.local_part == name.local_part && qname.url == name.url
                },
                Node::NodeText { sequence, content } => false,
                _ => panic!("error {:?}", node)
            }
        },
        _ => panic!("error {:?}", test)
    }
}


fn eval_predicates<'a>(exprs: Vec<Expr>, env: Box<Environment<'a>>, value: Object, context_item: &Object) -> EvalResult<'a> {
    let mut current_env = env;
    let mut result = value;

    for expr in exprs {
        match expr {
            Expr::Predicate(cond) => {
                match *cond {
                    Expr::Integer(pos) => {
                        let pos = pos;
                        if pos <= 0 {
                            result = Object::Empty
                        } else {
                            match result {
                                Object::Range { min, max } => {
                                    let len = max - min + 1;

                                    if pos > len {
                                        result = Empty;
                                    } else {
                                        let num = min + pos - 1;
                                        result = Object::Atomic(Type::Integer(num));
                                    }
                                },
                                Object::Sequence(items) => {
                                    result = if let Some(item) = items.get((pos - 1) as usize) {
                                        item.clone()
                                    } else {
                                        Object::Empty
                                    };
                                },
                                Object::Node(node) => {
                                    result = if pos == 1 {
                                        Object::Node(node)
                                    } else {
                                        Object::Empty
                                    }
                                }
                                _ => panic!("predicate {:?} on {:?}", pos, result)
                            }
                        }
                    },
                    Expr::Comparison { left, operator, right } => {
                        let it = object_to_iterator(&result);

                        let mut evaluated = vec![];
                        for item in it {
                            let context_item = item;

                            let (_, l_value) = eval_expr(*left.clone(), current_env.clone(), &context_item)?;
                            let (_, r_value) = eval_expr(*right.clone(), current_env.clone(), &context_item)?;

                            let check = match operator {
                                OperatorComparison::GeneralEquals => comparison::general_eq(&l_value, &r_value),
                                OperatorComparison::ValueEquals => comparison::eq(&l_value, &r_value),
                                OperatorComparison::ValueNotEquals => comparison::ne(&l_value, &r_value),
                                OperatorComparison::ValueLessThan => comparison::ls(&l_value, &r_value),
                                OperatorComparison::ValueLessOrEquals => comparison::ls_or_eq(&l_value, &r_value),
                                OperatorComparison::ValueGreaterThan => comparison::gr(&l_value, &r_value),
                                OperatorComparison::ValueGreaterOrEquals => comparison::gr_or_eq(&l_value, &r_value),
                                _ => panic!("operator {:?} is not implemented", operator)
                            };

                            match check {
                                Ok(true) => evaluated.push(context_item),
                                Err(code) => {
                                    return Err((code, String::from("TODO")));
                                },
                                _ => {}
                            }
                        }

                        if evaluated.len() == 0 {
                            result = Object::Empty;
                        } else if evaluated.len() == 1 {
                            result = evaluated[0].clone() //TODO: try to avoid clone here
                        } else {
                            result = Object::Sequence(evaluated)
                        }
                    }
                    _ => panic!("unknown {:?} {:?}", cond, result)
                }
            }
            _ => panic!("unknown {:?}", expr)
        }
    }

    Ok((current_env, result))
}

fn is_context_dependent(expression: &Expr) -> bool {
    if DEBUG {
        println!("is_context_dependent {:?}", expression);
    }
    match expression {
        Expr::ContextItem => true,
        _ => false
    }
}

pub struct RangeIterator {
    till: i128,
    next: i128,
    step: i128
}

impl RangeIterator {

    pub(crate) fn create(min: i128, max: i128) -> (Self, usize) {
        if min > max {
            (RangeIterator::new(min, -1, max), (min - max).min(0) as usize)
        } else {
            (RangeIterator::new(min, 1, max), (max - min).min(0) as usize)
        }
    }

    fn new(next: i128, step: i128, till: i128) -> Self {
        RangeIterator {
            till, next, step
        }
    }
}

impl Iterator for RangeIterator {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next;
        self.next = self.next + self.step;

        if (self.step > 0 && curr <= self.till) || (self.step < 0 && curr >= self.till) {
            Some(Object::Atomic(Type::Integer(curr)))
        } else {
            None
        }
    }
}

pub fn object_to_integer(object: Object) -> i128 {
    match object {
        Object::Atomic(Type::Integer(n)) => n,
        _ => panic!("TODO object_to_integer {:?}", object)
    }
}

// TODO: optimize!!!
pub fn object_to_iterator<'a>(object: &Object) -> Vec<Object> {
    match object {
        Object::Atomic(..) => {
            let mut result = Vec::with_capacity(1);
            result.push(object.clone());
            result
        },
        Object::Range { min , max } => {
            let (it, count) = RangeIterator::create(*min, *max);

            let mut result = Vec::with_capacity(count.min(0) as usize);
            for item in it {
                result.push(item);
            }
            result
        },
        Object::Array(items) => {
            items.clone() // optimize?
        },
        Object::Sequence(items) => {
            items.clone() // optimize?
        },
        _ => panic!("TODO object_to_iterator {:?}", object)
    }
}

// TODO: optimize!!!
pub fn object_owned_to_sequence<'a>(object: Object) -> Vec<Object> {
    // println!("object_to_iterator for {:?}", object);
    match object {
        Object::ForBinding { name, values } => {
            object_owned_to_sequence(*values)
        },
        Object::Empty |
        Object::Node(..) |
        Object::Atomic(..) => {
            let mut result = Vec::with_capacity(1);
            result.push(object);
            result
        },
        Object::Range { min , max } => {
            let (it, count) = RangeIterator::create(min, max);
            let mut result = Vec::with_capacity(count.min(0) as usize);
            for item in it {
                result.push(item);
            }
            result
        },
        Object::Array(items) => {
            items
        },
        Object::Sequence(items) => {
            items
        },
        _ => panic!("TODO object_to_iterator {:?}", object)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::*;

    #[test]
    fn eval1() {
        test_eval(
            "empty(<a/>/a)",
            Object::Atomic(Type::Boolean(true))
        )
    }

    #[test]
    fn eval2() {
        test_eval(
            "deep-equal(string-to-codepoints($result),
            (97, 10, 10, 10, 32, 10, 115, 116, 114, 105, 110, 103, 32, 108, 105, 116, 101, 114, 97, 108, 32, 10))",
            Object::Empty
        )
    }

    fn test_eval(input: &str, expected: Object) {
        let result = parse(input);

        if DEBUG {
            println!("parsed: {:?}", result);
        }

        if result.is_ok() {
            let program = result.unwrap();
            let mut env = Environment::new();

            let result = eval_statements(program, Box::new(env)).unwrap();

            assert_eq!(
                result,
                expected
            );
        } else {
            println!("parse error: {:?}", result);
            panic!("parse return error");
        }
    }
}
