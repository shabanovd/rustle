use crate::parser::Statement;
use crate::parser::Expr;
use crate::parser::Operator;

mod environment;
pub use self::environment::Environment;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Empty,

    Boolean(bool),
    Integer(i128),
    String(String),

    Function{parameters: Vec<String>, body: Vec<Statement>},

    Return(Box<Object>),
}

pub fn eval_statements(statements: Vec<Statement>, env: &mut Environment) -> Object {

    let mut result = Object::Empty;

    for statement in statements {
        result = eval_statement(statement, env);

        if let &Object::Return(_) = &result {
            return result;
        }
    }

    println!("result: {:?}", result);

    result
}

fn eval_statement(statement: Statement, env: &mut Environment) -> Object {
    match statement {
        Statement::Expression(expr) => eval_expr(expr, env),
        _ => panic!("TODO")
    }
}

fn eval_expr(expression: Expr, env: &mut Environment) -> Object {
    println!("eval_expr: {:?}", expression);

    match expression {
        Expr::Boolean(bool) => Object::Boolean(bool),
        Expr::Integer(number) => Object::Integer(number),
        Expr::String(string) => Object::String(string),

        Expr::Binary { left, operator: Operator::Multiply, right } => {
            match (eval_expr(*left, env), eval_expr(*right, env)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
                _ => panic!("multiply fail")
            }

        },

        Expr::Call {function, arguments} => {
            let (parameters, body) = match *function {
                Expr::QName { local_part, ns: _, prefix: _ } => {
                    let function_name = local_part; //TODO: fix it!!!
                    match env.get(&function_name) {
                        Some(Object::Function {parameters, body}) => (parameters, body),
                        None => {
                            let arguments = arguments.into_iter().map(|statement| eval_statement(statement, env)).collect();
                            return eval_builtin(&function_name, arguments).expect("error calling function");
                        }
                        _ => panic!("fail to call function"),
                    }
                }
                _ => panic!("fail to call function"),
            };

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {
                function_environment.set(parameter, eval_statement(argument, env));
            }

            eval_statements(body, &mut function_environment)

        }
        _ => panic!("TODO")
    }
}

fn eval_builtin(function_name: &str, arguments: Vec<Object>) -> Option<Object> {

    println!("eval_builtin: {:?} {:?}", function_name, arguments);

    match (function_name, arguments.as_slice()) {
        ("decimal", [Object::String(string)]) => Some(Object::Integer(string.parse::<i128>().unwrap())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn eval_simple() {
        test_eval("xs:decimal(\"617375191608514839\") * xs:decimal(\"0\")", Object::Integer(0))
    }


    fn test_eval(input: &str, expected: Object) {
        let result = parse(input);

        if result.is_ok() {
            let (_, program) = result.unwrap();
            let mut env = Environment::new();

            let result = eval_statements(program, &mut env);

            assert_eq!(
                expected,
                result
            );
        }
    }
}