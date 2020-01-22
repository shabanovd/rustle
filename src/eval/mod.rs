use crate::parser::Statement;
use crate::parser::Expr;
use crate::parser::Operator;

mod environment;
pub use self::environment::Environment;

use std::collections::HashMap;
use crate::eval::Object::Empty;

const DEBUG: bool = true;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
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

    Decimal(i128),
    Integer(i128),
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

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct QName {
    pub prefix: String,
    pub url: String,
    pub local_part: String,
}

impl QName {
    fn new(local_part: &str) -> Self {
        QName {
            prefix: String::from("" ),
            url: String::from("" ),
            local_part: String::from( local_part ),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Node {
    Node { name: QName, attributes: Vec<Node>, children: Vec<Node> },
    Attribute { name: QName, value: String },
    NodeText(String),
    NodeComment(String),
    NodePI { target: QName, content: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Empty,

    Range { from: Type, till: Type },
    Sequence(Vec<Object>),

    QName { prefix: String, url: String, local_part: String },

    Atomic(Type),
    Node(Node),

    Map(HashMap<Type, Object>),

    Function{parameters: Vec<String>, body: Vec<Statement>},

    Return(Box<Object>),
}

pub fn eval_statements<'a>(statements: Vec<Statement>, env: &'a mut Environment<'a>) -> (&'a mut Environment<'a>, Object) {

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

fn eval_statement<'a>(statement: Statement, env: &'a mut Environment<'a>) -> (&'a mut Environment<'a>, Object) {
    match statement {
        Statement::Expression(expr) => eval_expr(expr, env),
        _ => panic!("TODO: {:?}", statement)
    }
}

fn eval_expr<'a>(expression: Expr, env: &'a mut Environment<'a>) -> (&'a mut Environment<'a>, Object) {
    if DEBUG {
        println!("eval_expr: {:?}", expression);
    }

    let mut current_env = env;

    match expression {
        Expr::Boolean(bool) => (current_env, Object::Atomic(Type::Boolean(bool))),
        Expr::Integer(number) => (current_env, Object::Atomic(Type::Integer(number))),
        Expr::String(string) => (current_env, Object::Atomic(Type::String(string))),

        Expr::QName { local_part, url, prefix } => {
            (current_env, Object::QName { local_part, url, prefix })
        },

        Expr::Node { name, attributes , children } => {
            let (new_env, evaluated_name) = eval_expr(*name, current_env);
            current_env = new_env;

            let evaluated_name = match evaluated_name {
                Object::QName { local_part, url, prefix } => {
                    QName { local_part, url, prefix }
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            let mut evaluated_attributes = vec![];
            for attribute in attributes {
                let (new_env, evaluated_attribute) = eval_expr(attribute, current_env);
                current_env = new_env;

                match evaluated_attribute {
                    Object::Node(Node::Attribute { name, value}) => { // TODO: avoid copy!
                        let evaluated_attribute = Node::Attribute { name, value };
                        evaluated_attributes.push(evaluated_attribute);
                    }
                    _ => panic!("unexpected object") //TODO: better error
                };
            }

            let mut evaluated_children = vec![];
            for child in children {
                let (new_env, evaluated_child) = eval_expr(child, current_env);
                current_env = new_env;

                match evaluated_child {
                    Object::Node(Node::Attribute { name, value}) => { // TODO: avoid copy!
                        let evaluated_attribute = Node::Attribute { name, value };

                        evaluated_attributes.push(evaluated_attribute);
                    },
                    Object::Node(node) => {
                        evaluated_children.push(node);
                    }
                    _ => panic!("unexpected object") //TODO: better error
                };


            }

            (current_env, Object::Node(
                Node::Node { name: evaluated_name, attributes: evaluated_attributes, children: evaluated_children }
            ))
        },

        Expr::Attribute { name, value } => {
            let (new_env, evaluated_name) = eval_expr(*name, current_env);
            current_env = new_env;

            let evaluated_name = match evaluated_name {
                Object::QName { prefix, url, local_part } => { // TODO: avoid copy!
                    QName { prefix, url, local_part }
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            let (new_env, evaluated_value) = eval_expr(*value, current_env);
            current_env = new_env;

            let evaluated_value = match evaluated_value {
                Object::Atomic(Type::String(string)) => { // TODO: avoid copy!
                    string
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            (current_env, Object::Node(Node::Attribute { name: evaluated_name, value: evaluated_value }))
        },

        Expr::NodeText(content) => (current_env, Object::Node(Node::NodeText(content))),
        Expr::NodeComment(content) => (current_env, Object::Node(Node::NodeComment(content))),
        Expr::NodePI { target, content } => {
            let (new_env, evaluated_target) = eval_expr(*target, current_env);
            current_env = new_env;

            let evaluated_target = match evaluated_target {
                Object::QName { prefix, url, local_part } => { // TODO: avoid copy!
                    QName { prefix, url, local_part }
                }
                _ => panic!("unexpected object") //TODO: better error
            };

            (current_env, Object::Node(Node::NodePI { target: evaluated_target, content }))
        },

        Expr::Map { entries } => {
            let mut map = HashMap::new();
            for entry in entries {
                match entry {
                    Expr::MapEntry { key, value } => {
                        let (new_env, evaluated_key) = eval_statement(*key, current_env);
                        current_env = new_env;

                        let (new_env, evaluated_value) = eval_statement(*value, current_env);
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

        Expr::Binary { left, operator: Operator::Multiply, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env);
            current_env = new_env;

            let (new_env, right_result) = eval_expr(*right, current_env);
            current_env = new_env;

            if DEBUG {
                println!("left_result {:?}", left_result);
                println!("right_result {:?}", right_result);
            }

            let result = match (left_result, right_result) {
                (Object::Atomic(Type::Integer(left)), Object::Atomic(Type::Integer(right))) =>
                    Object::Atomic(Type::Integer(left * right)),

                _ => panic!("multiply fail")
            };

            (current_env, result)
        },

        Expr::Call {function, arguments} => {
            let (parameters, body) = match *function {
                Expr::QName { local_part, url, prefix: _ } => {
                    match current_env.get(&local_part) { //TODO: fix it!!!
                        Some(Object::Function {parameters, body}) => (parameters, body),
                        None => {
                            let mut evaluated_arguments = vec![];
                            for argument in arguments {
                                let (new_env, value) = eval_statement(argument, current_env);
                                current_env = new_env;

                                evaluated_arguments.push(value);
                            }

                            if DEBUG {
                                println!("eval_builtin: {:?} {:?}", &local_part, evaluated_arguments);
                            }

                            let fun = current_env.functions.get(&url, &local_part, evaluated_arguments.len());

                            return if fun.is_some() {
                                fun.unwrap()(current_env, evaluated_arguments)
                            } else {
                                //TODO: raise error
                                (current_env, Object::Empty)
                            }
                        }
                        _ => panic!("fail to call function"),
                    }
                }
                _ => panic!("fail to call function"),
            };

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {
                let (new_env, new_result) = eval_statement(argument, current_env);

                current_env = new_env;

                function_environment.set(parameter, new_result);
            }

            let (_, result) = eval_statements(body, &mut function_environment);

            (current_env, result)
        },

        Expr::Range { from, till } => {
            let (new_env, evaluated_from) = eval_expr(*from, current_env);
            current_env = new_env;

            let (new_env, evaluated_till) = eval_expr(*till, current_env);
            current_env = new_env;

            let from_type = match evaluated_from {
                Object::Atomic(t) => t,
                _ => panic!("from is not atomic")
            };

            let till_type = match evaluated_till {
                Object::Atomic(t) => t,
                _ => panic!("till is not atomic")
            };

            (current_env, Object::Range { from: from_type, till: till_type})
        }

        Expr::Postfix { primary, suffix } => {
            let (new_env, value) = eval_expr(*primary, current_env);
            current_env = new_env;

            let mut result = value;

            for suf in  suffix {
                match suf {
                    Expr::Predicate(cond) => {
                        match *cond {
                            Statement::Expression(Expr::Integer(pos)) => {

                                match result {
                                    Object::Range { from: from_type , till: till_type } => {
                                        let from = type_to_int(from_type);
                                        let till = type_to_int(till_type);

                                        let dist = till - from + 1;

                                        if pos > dist {
                                            result = Empty;
                                        } else {
                                            result = Object::Atomic(Type::Integer(from + pos - 1));
                                        }
                                    },
                                    _ => panic!("predicate {:?} on {:?}", pos, result)
                                }
                            }
                            _ => panic!("unknown suffix statement {:?} {:?}", cond, result)
                        }
                    }
                    _ => panic!("unknown suffix {:?}", suf)
                }
            }

            (current_env, result)
        }

        Expr::Sequence(exprs) => {
            if exprs.len() == 0 {
                (current_env, Object::Empty)
            } else if exprs.len() == 1 {
                let expr = exprs.get(0).unwrap().clone();

                let (new_env, value) = eval_statement(expr, current_env);
                current_env = new_env;

                (current_env, value)
            } else {
                let mut evaluated = vec![];
                for expr in exprs {
                    let (new_env, value) = eval_statement(expr, current_env);
                    current_env = new_env;

                    evaluated.push(value);
                }

                if evaluated.len() == 0 {
                    (current_env, Object::Empty)
                } else if evaluated.len() == 1 {
                    (current_env, evaluated[0].clone()) //TODO: try to avoid clone here
                } else {
                    (current_env, Object::Sequence(evaluated))
                }
            }
        },
        _ => panic!("TODO {:?}", expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn eval_decimal() {
        test_eval(
            "xs:decimal(\"617375191608514839\") * xs:decimal(\"0\")",
            Object::Atomic(Type::Integer(0)))
    }

    #[test]
    fn eval_map_get() {
        test_eval(
            "map:get(map{1:\"Sunday\",2:\"Monday\",3:\"Tuesday\",4:\"Wednesday\",5:\"Thursday\",6:\"Friday\",7:\"Saturday\"}, 4)",
            Object::Atomic(Type::String( String::from("Wednesday")))
        )
    }

    #[test]
    fn eval_sequence1() {
        test_eval(
            "(1 to 5)[10]",
            Object::Empty
        );

        test_eval(
            "(21 to 29)[5]",
            Object::Atomic(Type::Integer(25))
        );
    }

    #[test]
    fn eval_sequence2() {
        test_eval(
            "(1 to 100)[. mod 5 eq 0]",
            Object::Sequence([
                Object::Atomic(Type::Integer(5)),
                Object::Atomic(Type::Integer(10)),
                Object::Atomic(Type::Integer(15)),
                Object::Atomic(Type::Integer(20)),
                Object::Atomic(Type::Integer(25)),
                Object::Atomic(Type::Integer(30)),
                Object::Atomic(Type::Integer(35)),
                Object::Atomic(Type::Integer(40)),
                Object::Atomic(Type::Integer(45)),
                Object::Atomic(Type::Integer(50)),
                Object::Atomic(Type::Integer(55)),
                Object::Atomic(Type::Integer(60)),
                Object::Atomic(Type::Integer(65)),
                Object::Atomic(Type::Integer(70)),
                Object::Atomic(Type::Integer(75)),
                Object::Atomic(Type::Integer(80)),
                Object::Atomic(Type::Integer(85)),
                Object::Atomic(Type::Integer(90)),
                Object::Atomic(Type::Integer(95)),
            ].to_vec())
        );
    }

    #[test]
    fn eval_direct_node_creation() {
        test_eval(
            "<book isbn=\"isbn-0060229357\">\
    <title>Harold and the Purple Crayon</title>\
    <author>\
        <first>Crockett</first>\
        <last>Johnson</last>\
    </author>\
</book>",
            Object::Node(Node::Node {
                name: QName::new("book"),
                attributes: [
                    Node::Attribute { name: QName::new("isbn"), value: "isbn-0060229357".to_string() }
                ].to_vec(),
                children: [
                    Node::Node {
                        name: QName::new("title"),
                        attributes: Vec::new(),
                        children: [
                            Node::NodeText("Harold and the Purple Crayon".to_string())
                        ].to_vec()
                    },
                    Node::Node {
                        name: QName::new("author"),
                        attributes: Vec::new(),
                        children: [
                            Node::Node {
                                name: QName::new("first"),
                                attributes: Vec::new(),
                                children: [
                                    Node::NodeText("Crockett".to_string())
                                ].to_vec()
                            },
                            Node::Node {
                                name: QName::new("last"),
                                attributes: Vec::new(),
                                children: [
                                    Node::NodeText("Johnson".to_string())
                                ].to_vec()
                            }
                        ].to_vec()
                    }
                ].to_vec()
            }
        ))
    }

    fn test_eval(input: &str, expected: Object) {
        let result = parse(input);

        if DEBUG {
            println!("parsed: {:?}", result);
        }

        if result.is_ok() {
            let (_, program) = result.unwrap();
            let mut env = Environment::new();

            let (new_env, result) = eval_statements(program, &mut env);

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