use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use crate::eval::Object::Empty;
use crate::fns::{call, Param, sort_and_dedup};
use crate::parser::op::{Representation, Statement, Expr, Operator, ItemType};
use crate::value::{QName, QNameResolved, resolve_element_qname, resolve_function_qname};

pub use self::environment::Environment;
use crate::serialization::object_to_string;
use crate::serialization::to_string::{ref_to_string, object_to_string_xml};
use rust_decimal::Decimal;

mod environment;
pub(crate) mod comparison;

const DEBUG: bool = false;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum Type {
    dateTime(),
    dateTimeStamp(),

    date(),
    time(),

    duration(),
    yearMonthDuration(),
    dayTimeDuration(),

    float(),
    double(),

    Integer(i128),
    Decimal(Decimal),
    Double(Decimal),
    nonPositiveInteger(),
    negativeInteger(),
    long(),
    int(),
    short(),
    byte(),

    nonNegativeInteger(),
    unsignedLong(),
    unsignedInt(),
    unsignedShort(),
    unsignedByte(),

    positiveInteger(),

    gYearMonth(),
    gYear(),
    gMonthDay(),
    gDay(),
    gMonth(),

    // TODO CharRef { representation: Representation, reference: u32 }, ?
    String(String),
    NormalizedString(String),
    Token(String),
    language(String),
    NMTOKEN(String),
    Name(String),
    NCName(String),
    ID(String),
    IDREF(String),
    ENTITY(String),

    Boolean(bool),
    base64Binary(),
    hexBinary(),
    AnyURI(String),
    QName(),
    NOTATION(),
}

fn type_to_int(t: Type) -> i128 {
    match t {
        Type::Integer(num) => num,
        _ => panic!("can't convert to int {:?}", t)
    }
}

fn object_to_qname(t: Object) -> QName {
    match t {
        Object::QName { prefix, url, local_part } => QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", t)
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Node {
    Node { sequence: isize, name: QName, attributes: Vec<Node>, children: Vec<Node> },
    Attribute { sequence: isize, name: QName, value: String },
    NodeText { sequence: isize, content: String },
    NodeComment { sequence: isize, content: String },
    NodePI { sequence: isize, target: QName, content: String },
}

fn node_to_number(node: &Node) -> &isize {
    match node {
        Node::Node { sequence, .. } => sequence,
        Node::Attribute { sequence, .. } => sequence,
        Node::NodeText { sequence, .. } => sequence,
        Node::NodeComment { sequence, .. } => sequence,
        Node::NodePI { sequence, .. } => sequence,
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        node_to_number(self).cmp(node_to_number(other))
    }
}

impl PartialOrd<Self> for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        node_to_number(self).partial_cmp(node_to_number(other))
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let write = |f: &mut fmt::Formatter, qname: &QName| {
            if !qname.prefix.is_empty() {
                write!(f, "{}:", qname.prefix).unwrap();
            }
            write!(f, "{}", qname.local_part).unwrap();
        };

        match self {
            Node:: NodePI { sequence, target, content } => {
                write!(f, "<?")?;
                write(f, target);
                write!(f, "{:?}?>", content)?;
            },
            Node:: NodeComment {sequence, content} => {
                write!(f, "<!--{}-->", content)?;
            },
            Node:: NodeText { sequence, content} => {
                write!(f, "{}", content)?;
            },
            Node:: Attribute { sequence, name, value } => {
                write!(f, "@")?;
                write(f, name);
                write!(f, "={:?}", value)?;
            },
            Node::Node { sequence, name, attributes, children } => {
                write!(f, "<")?;

                write(f, name);

                if attributes.len() > 0 {
                    for attribute in attributes {
                        println!("attribute {:?}", attribute);
                        match attribute {
                            Node::Attribute { sequence, name, value } => {
                                write!(f, " ")?;
                                write(f, name);
                                write!(f, "={}", value)?;
                            },
                            _ => panic!("unexpected")
                        }
                    }
                }

                if children.len() == 0 {
                    write!(f, "/>")?;
                } else {
                    write!(f, ">").unwrap();
                    for child in children {
                        write!(f, "{:?}", child)?;
                    }
                    write!(f, "</")?;
                    write(f, name);
                }
            },
            _ => panic!("unexpected")
        }
        write!(f, "")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    // workaround
    Error { code: String },
    CharRef { representation: Representation, reference: u32 },
    EntityRef(String),
    
    Nothing,

    Empty,

    Range { min: i128, max: i128 },
    Sequence(Vec<Object>),

    QName { prefix: String, url: String, local_part: String },

    Atomic(Type),
    Node(Node),

    Array(Vec<Object>),
    Map(HashMap<Type, Object>),

    Function { parameters: Vec<Param>, body: Box<Expr> },
    FunctionRef { name: QNameResolved, arity: usize },

    Return(Box<Object>),
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.cmp(t2)
                    },
                    _ => Ordering::Greater,
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        n1.cmp(n2)
                    },
                    _ => Ordering::Less,
                }
            },
            _ => Ordering::Less
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Object) -> Option<Ordering> {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.partial_cmp(t2)
                    },
                    _ => Some(Ordering::Greater),
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        n1.partial_cmp(n2)
                    },
                    _ => Some(Ordering::Less),
                }
            },
            _ => Some(Ordering::Less)
        }
    }
}

pub fn eval_statements<'a>(statements: Vec<Statement>, env: Box<Environment<'a>>) -> (Box<Environment<'a>>, Object) {

    let mut result = Object::Empty;

    let mut current_env = env;

    for statement in statements {
        let (new_env, new_result) = eval_statement(statement, current_env);
        current_env = new_env;

        result = new_result;

        if let &Object::Return(_) = &result {
            return (current_env, result);
        }
    }

    if DEBUG {
        println!("result: {:?}", result);
    }

    (current_env, result)
}

fn eval_statement<'a>(statement: Statement, env: Box<Environment<'a>>) -> (Box<Environment<'a>>, Object) {
    match statement {
        Statement::Prolog(exprs) => (eval_prolog(exprs, env), Object::Nothing),
        Statement::Program(expr) => eval_expr(expr, env, &Object::Nothing),
        _ => panic!("TODO: {:?}", statement)
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

            current_env
        },
        _ => panic!("unexcpected at prolog {:?}", expression)
    }
}

pub fn eval_exprs<'a>(exprs: Vec<Expr>, env: Box<Environment<'a>>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut result = Object::Empty;

    let mut current_env = env;

    for expr in exprs {
        let (new_env, new_result) = eval_expr(expr, current_env, context_item);
        current_env = new_env;

        // TODO: review it
        result = new_result;

        if let &Object::Return(_) = &result {
            return (current_env, result);
        }
    }

    (current_env, result)
}

pub fn eval_expr<'a>(expression: Expr, env: Box<Environment<'a>>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    if DEBUG {
        println!("eval_expr: {:?}", expression);
    }

    let mut current_env = env;

    match expression {
        Expr::Boolean(bool) => (current_env, Object::Atomic(Type::Boolean(bool))),
        Expr::Integer(number) => (current_env, Object::Atomic(Type::Integer(number))),
        Expr::Decimal(number) => (current_env, Object::Atomic(Type::Decimal(number))),
        Expr::Double(number) => (current_env, Object::Atomic(Type::Double(number))),
        Expr::String(string) => (current_env, Object::Atomic(Type::String(string))),
        Expr::StringComplex(exprs) => {
            let mut strings = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let (new_env, object) = eval_expr(expr, current_env, context_item);
                current_env = new_env;

                let str = object_to_string(&object);
                strings.push(str);
            }

            (current_env, Object::Atomic(Type::String(strings.join(""))))
        }
        Expr::EscapeQuot => (current_env, Object::Atomic(Type::String(String::from("\"")))),
        Expr::EscapeApos => (current_env, Object::Atomic(Type::String(String::from("'")))),
        Expr::CharRef { representation, reference } => {
            (current_env, Object::CharRef { representation, reference })
        }
        Expr::EntityRef(reference) => {
            (current_env, Object::EntityRef(reference))
        }

        Expr::ContextItem => {
            // TODO: optimize to avoid clone if possible
            (current_env, context_item.clone())
        },

        Expr::QName { local_part, url, prefix } => {
            (current_env, Object::QName { local_part, url, prefix })
        },

        Expr::Body(exprs) => {
            if exprs.len() == 0 {
                (current_env, Object::Empty)
            } else if exprs.len() == 1 {
                let expr = exprs.get(0).unwrap().clone();

                let (new_env, value) = eval_expr(expr, current_env, context_item);
                current_env = new_env;

                (current_env, value)
            } else {
                let mut evaluated = vec![];
                for expr in exprs {
                    let (new_env, value) = eval_expr(expr, current_env, context_item);
                    current_env = new_env;

                    match value {
                        Object::Empty => {},
                        _ => evaluated.push(value)
                    }
                }

                if evaluated.len() == 0 {
                    (current_env, Object::Empty)
                } else if evaluated.len() == 1 {
                    (current_env, evaluated[0].clone()) //TODO: try to avoid clone here
                } else {
                    // TODO understand when it should happen... sort_and_dedup(&mut evaluated);
                    (current_env, Object::Sequence(evaluated))
                }
            }
        },

        Expr::Steps(steps) => {
            let mut current_context_item = context_item.clone();
            for step in steps {
                println!("step {:?}", step);

                let (new_env, value) = eval_expr(step, current_env, &current_context_item);
                current_env = new_env;

                current_context_item = value;
                println!("result {:?}", current_context_item);
            }

            (current_env, current_context_item)
        },
        Expr::Path { steps,  expr } => {
            eval_expr(*expr, current_env, context_item)
        }

        Expr::AxisStep { step, predicates } => {
            let (new_env, value) = eval_expr(*step, current_env, context_item);
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

        Expr::Node { name, attributes , children } => {
            // let (new_env, evaluated_name) = eval_expr(*name, current_env, context_item);
            // current_env = new_env;
            //
            // let evaluated_name = match evaluated_name {
            //     Object::QName { local_part, url, prefix } => {
            //         QName { local_part, url, prefix }
            //     }
            //     _ => panic!("unexpected object") //TODO: better error
            // };

            let mut evaluated_attributes = vec![];
            for attribute in attributes {
                let (new_env, evaluated_attribute) = eval_expr(attribute, current_env, context_item);
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
                let (new_env, evaluated_child) = eval_expr(child, current_env, context_item);
                current_env = new_env;

                match evaluated_child {
                    Object::Sequence(items) => {
                        for item in items {
                            let id = current_env.next_id();
                            match item {
                                Object::Node(Node::Attribute { sequence, name, value}) => { // TODO: avoid copy!
                                    let evaluated_attribute = Node::Attribute { sequence, name, value };

                                    evaluated_attributes.push(evaluated_attribute);
                                },
                                Object::Node(node) => {
                                    evaluated_children.push(node);
                                }
                                Object::Atomic(..) => {
                                    let content = object_to_string_xml(&item);
                                    evaluated_children.push(Node::NodeText { sequence: -1, content });
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
            (current_env, Object::Node(
                Node::Node { sequence: id, name, attributes: evaluated_attributes, children: evaluated_children }
            ))
        },

        Expr::Attribute { name, value } => {
            // let (new_env, evaluated_name) = eval_expr(*name, current_env, context_item);
            // current_env = new_env;
            //
            // let evaluated_name = match evaluated_name {
            //     Object::QName { prefix, url, local_part } => { // TODO: avoid copy!
            //         QName { prefix, url, local_part }
            //     }
            //     _ => panic!("unexpected object") //TODO: better error
            // };

            let (new_env, evaluated_value) = eval_expr(*value, current_env, context_item);
            current_env = new_env;

            let evaluated_value = match evaluated_value {
                Object::Atomic(Type::String(string)) => { // TODO: avoid copy!
                    string
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            let id = current_env.next_id();

            (current_env, Object::Node(Node::Attribute { sequence: id, name, value: evaluated_value }))
        },

        Expr::NodeText(content) => {
            let id = current_env.next_id();
            (current_env, Object::Node(Node::NodeText { sequence: id, content }))
        },
        Expr::NodeComment(content) => {
            let id = current_env.next_id();
            (current_env, Object::Node(Node::NodeComment { sequence: id, content }))
        },
        Expr::NodePI { target, content } => {
            // let (new_env, evaluated_target) = eval_expr(*target, current_env, context_item);
            // current_env = new_env;
            //
            // let evaluated_target = match evaluated_target {
            //     Object::QName { prefix, url, local_part } => { // TODO: avoid copy!
            //         QName { prefix, url, local_part }
            //     }
            //     _ => panic!("unexpected object") //TODO: better error
            // };

            let id = current_env.next_id();
            (current_env, Object::Node(Node::NodePI { sequence: id, target, content }))
        },

        Expr::Map { entries } => {
            let mut map = HashMap::new();
            for entry in entries {
                match entry {
                    Expr::MapEntry { key, value } => {
                        let (new_env, evaluated_key) = eval_expr(*key, current_env, context_item);
                        current_env = new_env;

                        let (new_env, evaluated_value) = eval_expr(*value, current_env, context_item);
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

            (current_env, Object::Map(map))
        },

        Expr::SimpleMap(exprs)  => {
            let mut result = Object::Empty;
            for (i, expr) in exprs.iter().enumerate() {
                if i == 0 {
                    let (new_env, evaluated) = eval_expr(expr.clone(), current_env, context_item);
                    current_env = new_env;

                    result = evaluated;
                } else {
                    let mut sequence = vec![];

                    let it = object_to_iterator(&result);
                    for item in it {
                        let (new_env, evaluated) = eval_expr(expr.clone(), current_env, &item);
                        current_env = new_env;

                        sequence.push(evaluated);
                    }

                    result = Object::Sequence(sequence);
                }
            }
            (current_env, result)
        },

        Expr::Negative(expr) => {
            let (new_env, result) = eval_expr(*expr, current_env, context_item);
            current_env = new_env;

            let result = match result {
                Object::Atomic(Type::Integer(num)) => Object::Atomic(Type::Integer(-num)),
                Object::Atomic(Type::Decimal(num)) => Object::Atomic(Type::Decimal(-num)),
                Object::Atomic(Type::Double(num)) => Object::Atomic(Type::Double(-num)),
                _ => panic!("Negative is unimplemented for {:?}", result)
            };

            (current_env, result)
        },
        Expr::Binary { left, operator, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env, context_item);
            current_env = new_env;

            let (new_env, right_result) = eval_expr(*right, current_env, context_item);
            current_env = new_env;

            if DEBUG {
                println!("left_result {:?}", left_result);
                println!("right_result {:?}", right_result);
            }

            let result = match operator {
                Operator::Plus => {
                    match (left_result, right_result) {
                        (Object::Atomic(Type::Integer(left)), Object::Atomic(Type::Integer(right))) =>
                            Object::Atomic(Type::Integer(left + right)),

                        _ => panic!("plus fail")
                    }
                },
                Operator::Multiply => {
                    match (left_result, right_result) {
                        (Object::Atomic(Type::Integer(left)), Object::Atomic(Type::Integer(right))) =>
                            Object::Atomic(Type::Integer(left * right)),

                        _ => panic!("multiply fail")
                    }
                },
                Operator::Mod => {
                    match (left_result, right_result) {
                        (Object::Atomic(Type::Integer(left)), Object::Atomic(Type::Integer(right))) =>
                            Object::Atomic(Type::Integer(left % right)),

                        _ => panic!("multiply fail")
                    }
                },
                _ => panic!("operator {:?} unimplemented", operator)
            };


            (current_env, result)
        },
        Expr::Comparison { left, operator, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env, context_item);
            current_env = new_env;

            let (new_env, right_result) = eval_expr(*right, current_env, context_item);
            current_env = new_env;

            // if DEBUG {
            println!("left_result {:?}", left_result);
            println!("right_result {:?}", right_result);
            // }

            let result = match operator {
                Operator::Equals => {
                    let flag = comparison::eq(&left_result, &right_result);
                    Object::Atomic(Type::Boolean(flag))
                },
                _ => panic!("operator {:?} unimplemented", operator)
            };

            (current_env, result)
        },

        Expr::Call {function, arguments} => {
            let name = resolve_function_qname(function, &current_env);

            let (parameters, body) = match current_env.get(&name ) {
                Some(Object::Function {parameters, body}) => (parameters, body),
                None => {
                    let mut evaluated_arguments = vec![];
                    for argument in arguments {
                        let (new_env, value) = eval_expr(argument, current_env, context_item);
                        current_env = new_env;

                        evaluated_arguments.push(value);
                    }

                    return call(current_env, name, evaluated_arguments, context_item)
                }
                _ => panic!("error")
            };

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {
                let (new_env, new_result) = eval_expr(argument, current_env, context_item);
                current_env = new_env;

                let name = resolve_function_qname(parameter.name, &current_env);

                function_environment.set(name, new_result);
            }

            let (_, result) = eval_expr(*body, Box::new(function_environment), context_item);

            (current_env, result)
        },

        Expr::Range { from, till } => {
            let (new_env, evaluated_from) = eval_expr(*from, current_env, context_item);
            current_env = new_env;

            let (new_env, evaluated_till) = eval_expr(*till, current_env, context_item);
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
                (current_env, Object::Empty)
            } else if min == max {
                (current_env, Object::Atomic(Type::Integer(min)))
            } else {
                (current_env, Object::Range { min, max })
            }
        },

        Expr::SquareArrayConstructor { items } => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                let (new_env, evaluated) = eval_expr(item, current_env, context_item);
                current_env = new_env;

                values.push(evaluated);
            }

            (current_env, Object::Array(values))
        },

        Expr::CurlyArrayConstructor(expr) => {
            let (new_env, evaluated) = eval_expr(*expr, current_env, context_item);
            current_env = new_env;

            let values = match evaluated {
                Object::Empty => vec![],
                _ => panic!("can't convert to array {:?}", evaluated)
            };

            (current_env, Object::Array(values))
        },

        Expr::Postfix { primary, suffix } => {
            let (new_env, value) = eval_expr(*primary, current_env, context_item);
            current_env = new_env;

            eval_predicates(suffix, current_env, value, context_item)
        },

        Expr::SequenceEmpty() => {
            (current_env, Object::Empty)
        },
        Expr::Sequence(expr) => {
            let (new_env, value) = eval_expr(*expr, current_env, context_item);
            current_env = new_env;

            println!("{:?}", value);

            (current_env, value)
        },

        Expr::Or(exprs) => {
            if exprs.len() == 0 {
                (current_env, Object::Empty)
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item);
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                if sequence.len() == 0 {
                    (current_env, Object::Empty)
                } else if sequence.len() == 1 {
                    let object = sequence.remove(0);
                    (current_env, object)
                } else {
                    let result = sequence.into_iter()
                        .map(|item| object_to_bool(&item))
                        .fold(true, |acc, value| acc || value );

                    (current_env, Object::Atomic(Type::Boolean(result)))
                }
            }
        },
        Expr::And(exprs) => {
            if exprs.len() == 0 {
                (current_env, Object::Empty)
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item);
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                if sequence.len() == 0 {
                    (current_env, Object::Empty)
                } else if sequence.len() == 1 {
                    let object = sequence.remove(0);
                    (current_env, object)
                } else {
                    let result = sequence.into_iter()
                        .map(|item| object_to_bool(&item))
                        .fold(true, |acc, value| acc && value );

                    (current_env, Object::Atomic(Type::Boolean(result)))
                }
            }
        },
        Expr::StringConcat(exprs) => {
            if exprs.len() == 0 {
                (current_env, Object::Empty)
            } else {
                let mut sequence = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let (new_env, evaluated) = eval_expr(expr, current_env, context_item);
                    current_env = new_env;

                    sequence.push(evaluated);
                }

                if sequence.len() == 0 {
                    (current_env, Object::Empty)
                } else if sequence.len() == 1 {
                    let object = sequence.remove(0);
                    (current_env, object)
                } else {
                    let result = sequence.into_iter()
                        .map(|item| object_to_string(&item))
                        .collect();

                    (current_env, Object::Atomic(Type::String(result)))
                }
            }
        },

        Expr::NamedFunctionRef { name, arity } => {
            let (new_env, arity) = eval_expr(*arity, current_env, context_item);
            current_env = new_env;

            let arity = object_to_integer(arity);
            // TODO: check arity value
            let arity = arity as usize;

            let name = resolve_function_qname(name, &current_env);

            (current_env, Object::FunctionRef { name, arity })
        },

        Expr::Function { arguments, body } => {
            (current_env, Object::Function { parameters: arguments, body })
        },

        Expr::FLWOR { clauses, return_expr } => {
            // TODO: new env?
            // TODO: handle  WhereClause | GroupByClause | OrderByClause | CountClause
            let (new_env, _) = eval_exprs(clauses, current_env, context_item);
            current_env = new_env;

            // println!("returnExpr {:#?}", returnExpr);

            let (new_env, evaluated) = eval_expr(*return_expr, current_env, context_item);
            current_env = new_env;

            (current_env, evaluated)
        },
        Expr::LetClause { bindings } => {
            for binding in bindings {
                let (new_env, _) = eval_expr(binding, current_env, context_item);
                current_env = new_env;
            }

            (current_env, Object::Empty)
        },
        Expr::LetBinding { name, type_declaration,  value } => {
            let (_, evaluated_value) = eval_expr(*value, current_env.clone(), context_item);

            // TODO: handle typeDeclaration

            let name = resolve_element_qname(name, &current_env);

            let mut new_env = *current_env.clone();
            new_env.set(name, evaluated_value);

            (Box::new(new_env), Object::Empty)
        },
        Expr::VarRef { name } => {

            let name = resolve_element_qname(name, &current_env);

            if let Some(value) = current_env.get(&name) {
                (current_env, value)
            } else {
                panic!("unknown variable {:?}", name)
            }
        },

        Expr::TreatExpr { expr, st } => {
            let (new_env, object) = eval_expr(*expr, current_env, context_item);
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
                        Object::Atomic(Type::String(..)) => {
                            name.local_part == "string" && name.prefix == "xs" // TODO name.url ==
                        },
                        Object::Atomic(Type::NormalizedString(..)) => {
                            name.local_part == "string" && name.prefix == "xs" // TODO name.url ==
                        },
                        Object::Atomic(Type::Integer(..)) => {
                            name.local_part == "integer" && name.prefix == "xs" // TODO name.url ==
                        },
                        Object::Atomic(Type::Decimal(..)) => {
                            name.local_part == "decimal" && name.prefix == "xs" // TODO name.url ==
                        },
                        Object::Atomic(Type::Double(..)) => {
                            name.local_part == "double" && name.prefix == "xs" // TODO name.url ==
                        },
                        _ => panic!("TODO: {:?}", object)
                    }
                },
                _ => panic!("TODO: {:?}", item_type)
            };

            (current_env, Object::Atomic(Type::Boolean(result)))
        }

        _ => panic!("TODO {:?}", expression)
    }
}

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

fn step_and_test<'a>(step: Axis, test: Expr, env: Box<Environment<'a>>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
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
                                (env, Object::Empty)
                            } else if result.len() == 1 {
                                (env, result.remove(0))
                            } else {
                                (env, Object::Sequence(result))
                            }
                        },
                        _ => todo!()
                    }
                },
                _ => (env, Object::Empty)
            }
        },
        _ => (env, Object::Empty)
    }
}

fn test_node(test: &Expr, node: &Node) -> bool {
    match test {
        Expr::NameTest(qname) => {
            match node {
                Node::Node { sequence, name, attributes, children } => {
                    qname.local_part == name.local_part && qname.url == name.url
                },
                Node::NodeText { sequence, content } => false,
                _ => panic!("error {:?}", node)
            }
        },
        _ => panic!("error {:?}", test)
    }
}


fn eval_predicates<'a>(exprs: Vec<Expr>, env: Box<Environment<'a>>, value: Object, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    let mut current_env = env;
    let mut result = value;

    for expr in exprs {
        match expr {
            Expr::Predicate(cond) => {
                match *cond {
                    Expr::Integer(pos) => {
                        let pos = pos;
                        match result {
                            Object::Range { min , max } => {
                                let len = max - min + 1;

                                if pos > len {
                                    result = Empty;
                                } else {
                                    let num = min + pos - 1;
                                    result = Object::Atomic(Type::Integer(num));
                                }
                            },
                            _ => panic!("predicate {:?} on {:?}", pos, result)
                        }
                    },
                    Expr::Comparison { left, operator, right } => {
                        let it = object_to_iterator(&result);

                        let mut evaluated = vec![];
                        for item in it {
                            let context_item = item;

                            let (_, l_value) = eval_expr(*left.clone(), current_env.clone(), &context_item);
                            let (_, r_value) = eval_expr(*right.clone(), current_env.clone(), &context_item);

                            match operator {
                                Operator::Equals => {
                                    if l_value == r_value {
                                        evaluated.push(context_item)
                                    }
                                }
                                _ => panic!("operator {:?} is not implemented", operator)
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

    (current_env, result)
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

pub fn object_to_bool(object: &Object) -> bool {
    match object {
        Object::Empty => false,
        Object::Atomic(Type::Boolean(v)) => *v,
        _ => panic!("TODO object_to_bool {:?}", object)
    }
}

pub fn object_to_integer(object: Object) -> i128 {
    match object {
        Object::Atomic(Type::Integer(n)) => n,
        _ => panic!("TODO object_to_integer {:?}", object)
    }
}

pub fn object_to_iterator<'a>(object: &Object) -> Vec<Object> {
    // println!("object_to_iterator for {:?}", object);
    match object {
        Object::Range { min , max } => {
            // TODO: optimize!!!
            let (it, count) = if min > max {
                (RangeIterator::new(*min, -1, *max), *min - *max)
            } else {
                (RangeIterator::new(*min, 1, *max), *max - *min)
            };

            let mut result = Vec::with_capacity(count.min(0) as usize);
            for item in it {
                result.push(item);
            }
            result
        },
        Object::Array(items) => {
            items.clone() // optimize?
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

            let (new_env, result) = eval_statements(program, Box::new(env));

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
