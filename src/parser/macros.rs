
#[macro_export]
macro_rules! parse_sequence {
    ($fn_name:ident, $tag:expr, $parser_fn:ident, $expr_name:ident) => {
        fn $fn_name(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
            let (input, expr) = $parser_fn(input)?;

            let mut exprs = Vec::new();
            exprs.push(expr);

            let mut current_input = input;
            loop {
                let check = ws_tag($tag, current_input);
                if check.is_ok() {
                    let (input, _) = check?;
                    current_input = input;

                    let (input, expr) = $parser_fn(current_input)?;
                    current_input = input;

                    exprs.push(expr);
                } else {
                    break
                }
            }

            if exprs.len() == 1 {
                let expr = exprs.remove(0);
                Ok((
                    current_input,
                    expr
                ))
            } else {
                Ok((
                    current_input,
                    Expr::$expr_name(exprs)
                ))
            }
        }
    }
}

#[macro_export]
macro_rules! parse_surroundings {
    ($fn_name:ident, $begin:expr, $sep:expr, $end:expr, $parser_fn:ident, $expr_name:ident) => {
        fn $fn_name(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
            let (input, _) = ws_tag($begin, input)?;

            let (input, expr) = $parser_fn(input)?;

            let mut current_input = input;

            let check = ws_tag($sep, current_input);
            if check.is_err() {
                let (input, _) = ws_tag($end, current_input)?;
                current_input = input;

                Ok((
                    current_input,
                    expr
                ))
            } else {

                let mut exprs = Vec::new();
                exprs.push(expr);

                let mut current_input = input;
                loop {
                    let check = ws_tag($sep, current_input);
                    if check.is_ok() {
                        let (input, _) = check?;
                        current_input = input;

                        let (input, expr) = $parser_fn(current_input)?;
                        current_input = input;

                        exprs.push(expr);
                    } else {
                        break
                    }
                }

                let (input, _) = ws_tag($end, current_input)?;
                current_input = input;

                Ok((
                    current_input,
                    Expr::$expr_name(exprs)
                ))
            }
        }
    }
}

#[macro_export]
macro_rules! parse_one_of {
    ( $fn_name:ident, $($parser_fn:ident,)+ ) => {
        fn $fn_name(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
            $(
                let result = $parser_fn(input);
                match result {
                    Ok(..) => {
                        return result
                    }
                    Err(nom::Err::Failure(..)) => {
                        return result
                    }
                    _ => {}
                }
            )*
            result // TODO better error
        }
    }
}