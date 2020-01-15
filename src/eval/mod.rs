use crate::parser::Statement;
use crate::parser::Expr;
use crate::parser::Operator;
use crate::fns::FUNCTION;

mod environment;
pub use self::environment::Environment;
use crate::eval::Object::Empty;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Empty,

    Boolean(bool),
    Integer(i128),
    String(String),

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

    println!("result: {:?}", result);

    (current_env, result)
}

fn eval_statement<'a>(statement: Statement, env: &'a mut Environment<'a>) -> (&'a mut Environment<'a>, Object) {
    match statement {
        Statement::Expression(expr) => eval_expr(expr, env),
        _ => panic!("TODO")
    }
}

fn eval_expr<'a>(expression: Expr, env: &'a mut Environment<'a>) -> (&'a mut Environment<'a>, Object) {
    println!("eval_expr: {:?}", expression);

    let mut current_env = env;

    match expression {
        Expr::Boolean(bool) => (current_env, Object::Boolean(bool)),
        Expr::Integer(number) => (current_env, Object::Integer(number)),
        Expr::String(string) => (current_env, Object::String(string)),

        Expr::Binary { left, operator: Operator::Multiply, right } => {
            let (new_env, left_result) = eval_expr(*left, current_env);
            current_env = new_env;

            let (new_env, right_result) = eval_expr(*right, current_env);
            current_env = new_env;

            println!("left_result {:?}", left_result);
            println!("right_result {:?}", right_result);

            let result = match (left_result, right_result) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
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

                            println!("eval_builtin: {:?} {:?}", &local_part, evaluated_arguments);

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
        }
        _ => panic!("TODO")
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

            let (new_env, result) = eval_statements(program, &mut env);

            assert_eq!(
                expected,
                result
            );
        }
    }
}