use crate::parse_one_of;
use crate::parse_sequence;
use crate::parse_surroundings;
use crate::parser::op;
use crate::parser::errors::CustomError;

use nom::{branch::alt, bytes::complete::tag, character::complete::one_of, error::Error, IResult};

use crate::parser::helper::*;
use crate::fns::expr_to_params;
use crate::value::QName;
use crate::parser::parse_literal::{parse_literal, parse_integer_literal};
use crate::parser::parse_xml::parse_node_constructor;
use crate::parser::parse_names::{parse_eqname, parse_ncname};
use crate::parser::op::{Expr, found_expr, Statement, found_exprs, ItemType, OccurrenceIndicator, OperatorComparison, OperatorArithmetic};
use nom::sequence::{preceded, delimited};
use nom::combinator::opt;

const DEBUG: bool = false;

// TODO [2]    	VersionDecl 	   ::=    	"xquery" (("encoding" StringLiteral) | ("version" StringLiteral ("encoding" StringLiteral)?)) Separator

// [3]    	MainModule 	   ::=    	Prolog QueryBody
pub fn parse_main_module(input: &str) -> IResult<&str, Vec<Statement>, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, prolog) = parse_prolog(input)?;

    let (input, _) = ws(input)?;
    let (input, program) = parse_expr(input)?;

    Ok((input, vec![Statement::Prolog(prolog), Statement::Program(program)]))
}

// [6]    	Prolog 	   ::=
// TODO: ((DefaultNamespaceDecl | Setter | NamespaceDecl | Import) Separator)*
// TODO: ((ContextItemDecl | AnnotatedDecl | OptionDecl) Separator)*
// [7]    	Separator 	   ::=    	";"
pub fn parse_prolog(input: &str) -> IResult<&str, Vec<Expr>, CustomError<&str>> {

    let mut prolog = vec![];

    let mut current_input = input;
    loop {
        let check = parse_annotated_decl(current_input);
        if check.is_ok() {
            let (input, expr) = check?;
            let (input, _) = tag(";")(input)?;
            current_input = input;

            prolog.push(expr);
        } else {
            break
        }
    }

    found_exprs(current_input, prolog)
}

// [26]    	AnnotatedDecl 	   ::=    	"declare" Annotation* (VarDecl | FunctionDecl)
pub fn parse_annotated_decl(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let (input, _) = ws_tag("declare", input)?;
    let mut current_input = input;

    let mut annotations = vec![];
    loop {
        let check = parse_annotation(current_input);
        if check.is_ok() {
            let (input, annotation) = check?;
            current_input = input;

            annotations.push(annotation);
        } else {
            break
        }
    }

    let check = parse_var_decl(current_input);
    let (input, decl) = if check.is_ok() {
        let (input, decl) = check?;

        (input, Box::new(decl))
    } else {
        let (input, decl) = parse_function_decl(current_input)?;

        (input, Box::new(decl))
    };

    found_expr(input, Expr::AnnotatedDecl { annotations, decl } )
}

// [27]    	Annotation 	   ::=    	"%" EQName ("(" Literal ("," Literal)* ")")?
pub fn parse_annotation(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let (input, _) = ws_tag("%", input)?;

    let (input, name) = parse_eqname(input)?;

    let check = parse_annotation_value(input);
    if check.is_ok() {
        let (input, list) = check?;
        todo!()
    } else {
        found_expr(input, Expr::Annotation { name, value: None })
    }
}

parse_surroundings!(parse_annotation_value, "(", ",", ")", parse_literal, Literals);

// [28]    	VarDecl 	   ::=    	"variable" "$" VarName TypeDeclaration? ((":=" VarValue) | ("external" (":=" VarDefaultValue)?))
// [132]    	VarName 	   ::=    	EQName
pub fn parse_var_decl(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("variable", input)?;

    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    let mut current_input = input;

    let check = parse_type_declaration(current_input);
    let type_declaration = if check.is_ok() {
        let (input, td) = check?;
        current_input = input;
        Some(td)
    } else {
        None
    };

    let mut external = false;

    let check = ws_tag("external", current_input);
    if check.is_ok() {
        let (input, _) = check?;
        current_input = input;
        external = true;
    }

    let check = ws_tag(":=", current_input);
    let (input, value) = if check.is_ok() {
        let (input, _) = check?;
        current_input = input;

        let (input, expr) = parse_expr_single(current_input)?;
        (input, Some(expr))
    } else {
        if external {
            (current_input, None)
        } else {
            // TODO: is it correct?
            // return Err(nom::Err::Error(nom::error::ParseError::from_char(current_input, ' ')));
            todo!()
        }
    };

    found_expr(
        input,
        Expr::VarDecl {
            external, name,
            type_declaration: Box::new(type_declaration),
            value: Box::new(value)
        }
    )
}

// [33]    	ParamList 	   ::=    	Param ("," Param)*
parse_sequence!(parse_param_list, ",", parse_param, ParamList);

// [34]    	Param 	   ::=    	"$" EQName TypeDeclaration?
fn parse_param(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = tag("$")(input)?;
    let (input, name) = parse_eqname(input)?;

    let check = parse_type_declaration(input);
    let (input, td) = if check.is_ok() {
        let (input, expr) = check?;
        (input, Some(expr))
    } else {
        (input, None)
    };

    found_expr(
        input,
        Expr::Param { name, type_declaration: Box::new(td)}
    )
}

// [32]    	FunctionDecl 	   ::=    	"function" EQName "(" ParamList? ")" ("as" SequenceType)? (FunctionBody | "external")
// [35]    	FunctionBody 	   ::=    	EnclosedExpr
fn parse_function_decl(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws1_tag_ws1("function", input)?;

    let (input, name) = parse_eqname(input)?;

    let (input, _) = ws_tag_ws("(", input)?;

    let mut current_input = input;

    let check = parse_param_list(current_input);
    let params = if check.is_ok() {
        let (input, params) = check?;
        current_input = input;

        expr_to_params(params)
    } else {
        vec![]
    };

    let (input, _) = ws_tag(")", current_input)?;
    current_input = input;

    let check = parse_type_declaration(current_input);
    let type_declaration = if check.is_ok() {
        let (input, td) = check?;
        current_input = input;

        Box::new(Some(td))
    } else {
        Box::new(None)
    };

    let check = ws1_tag_ws1("external", current_input);
    let (input, external, body) = if check.is_ok() {
        let (input, _) = check?;

        (input, true, None)
    } else {
        let (input, body) = parse_enclosed_expr(current_input)?;
        (input, false, Some(Box::new(body)))
    };

    found_expr(input, Expr::FunctionDecl { name, params, external, type_declaration, body })
}

// [36]    	EnclosedExpr 	   ::=    	"{" Expr? "}"
pub(crate) fn parse_enclosed_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag_ws("{", input)?;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        check?
    } else {
        (input, Expr::Body(vec![]))
    };

    let (input, _) = ws_tag("}", input)?;

    found_expr(input, expr)
}

// [38]    	QueryBody 	   ::=    	Expr
// [39]    	Expr 	   ::=    	ExprSingle ("," ExprSingle)*
pub(crate) fn parse_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let mut program = vec![];

    let mut current_input = input;
    loop {
        let (input, expr) = parse_expr_single(current_input)?;

        program.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                found_expr(input, Expr::Body(program))
        }
        current_input = tmp?.0;
    }
}

// [40]    	ExprSingle 	   ::=    	FLWORExpr
//  TODO: | QuantifiedExpr
//  TODO: | SwitchExpr
//  TODO: | TypeswitchExpr
// | IfExpr
//  TODO: | TryCatchExpr
// | OrExpr
parse_one_of!(
    parse_expr_single, Expr,
    parse_flwor_expr,
    parse_if_expr,
    parse_or_expr,
);

// [41]    	FLWORExpr 	   ::=    	InitialClause IntermediateClause* ReturnClause
// [69]    	ReturnClause 	   ::=    	"return" ExprSingle
fn parse_flwor_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let mut clauses = vec![];

    let (input, initial_clause) = parse_initial_clause(input)?;
    let mut current_input = input;

    clauses.push(initial_clause);

    loop {
        let check = parse_intermediate_clause(current_input);
        if check.is_ok() {
            let (input, intermediate_claus) = check?;
            current_input = input;

            clauses.push(intermediate_claus);
        } else {
            break
        }
    }

    let (input, _) = ws1_tag_ws1("return", current_input)?;
    current_input = input;

    let (input, return_expr) = parse_expr_single(current_input)?;
    current_input = input;

    found_expr(
        current_input,
        Expr::FLWOR { clauses, return_expr: Box::new(return_expr) }
    )
}

// [42]    	InitialClause 	   ::=    	ForClause | LetClause | TODO WindowClause
parse_one_of!(
    parse_initial_clause, Expr,
    parse_for_clause,
    parse_let_clause,
);

// [43]    	IntermediateClause 	   ::=    	InitialClause | TODO WhereClause | GroupByClause | OrderByClause | CountClause
fn parse_intermediate_clause(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_initial_clause(input)
}

// [44]    	ForClause 	   ::=    	"for" ForBinding ("," ForBinding)*
fn parse_for_clause(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("for", input)?;

    let mut current_input = input;

    let mut bindings = vec![];
    loop {
        let (input, expr) = parse_for_binding(current_input)?;

        bindings.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                found_expr(
                    input,
                    Expr::ForClause { bindings }
                )
        }
        current_input = tmp?.0;
    }
}

// [45]    	ForBinding 	   ::=    	"$" VarName TypeDeclaration? AllowingEmpty? PositionalVar? "in" ExprSingle
fn parse_for_binding(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    // let check = parse_type_declaration(input);
    // TODO TypeDeclaration? AllowingEmpty? PositionalVar?

    let (input, _) = ws_tag("in", input)?;

    let (input, values) = parse_expr_single(input)?;

    found_expr(input, Expr::ForBinding { name, values: Box::new(values) })
}

// [48]    	LetClause 	   ::=    	"let" LetBinding ("," LetBinding)*
fn parse_let_clause(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("parse_let_clause_expr {:?}", input);
    }
    let (input, _) = ws_tag("let", input)?;
    let mut current_input = input;

    let mut bindings = vec![];
    loop {
        let (input, expr) = parse_let_binding(current_input)?;

        bindings.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                found_expr(
                    input,
                    Expr::LetClause { bindings }
                )
        }
        current_input = tmp?.0;
    }
}

// [49]    	LetBinding 	   ::=    	"$" VarName TypeDeclaration? ":=" ExprSingle
// [132]    	VarName 	   ::=    	EQName
fn parse_let_binding(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    let check = parse_type_declaration(input);
    let (input, td) = if check.is_ok() {
        let (input, td) = check?;
        (input, Some(td))
    } else {
        (input, None)
    };

    let (input, _) = ws_tag(":=", input)?;

    let (input, value) = parse_expr_single(input)?;

    found_expr(
        input,
        Expr::LetBinding { name, type_declaration: Box::new(td),  value: Box::new(value)}
    )
}

// [77]    	IfExpr 	   ::=    	"if" "(" Expr ")" "then" ExprSingle "else" ExprSingle
fn parse_if_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("if", input)?;

    let (input, _) = ws_tag("(", input)?;

    let (input, condition) = parse_expr(input)?;

    let (input, _) = ws_tag(")", input)?;

    let (input, _) = ws_tag("then", input)?;

    let (input, consequence) = parse_expr_single(input)?;

    let (input, _) = ws_tag("else", input)?;

    let (input, alternative) = parse_expr_single(input)?;

    found_expr(
        input,
        Expr::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: Box::new(alternative)
        }
    )

}

// [83]    	OrExpr 	   ::=    	AndExpr ( "or" AndExpr )*
parse_sequence!(parse_or_expr, "or", parse_and_expr, Or);

// [84]    	AndExpr 	   ::=    	ComparisonExpr ( "and" ComparisonExpr )*
parse_sequence!(parse_and_expr, "and", parse_comparison_expr, And);

// [85]    	ComparisonExpr 	   ::=    	StringConcatExpr ( ( ValueComp
// | GeneralComp
// | NodeComp) StringConcatExpr )?
fn parse_comparison_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, left) = parse_string_concat_expr(input)?;

    let current_input = input;

    let check = delimited(
        ws1,
        alt((
            tag("="), tag("!="), tag("<"), tag("<="), tag(">"), tag(">="),
            tag("eq"), tag("ne"), tag("lt"), tag("le"), tag("gt"), tag("ge"),
            // TODO: tag("is"), tag("<<"), tag(">>"),
        )),
        ws1
    )(current_input);
    if check.is_ok() {
        let (input, op) = check?;

        let (input, right) = parse_string_concat_expr(input)?;

        let operator = match op {
            "=" => OperatorComparison::GeneralEquals,
            "!=" => OperatorComparison::GeneralNotEquals,
            "<" => OperatorComparison::GeneralLessThan,
            "<=" => OperatorComparison::GeneralLessOrEquals,
            ">" => OperatorComparison::GeneralGreaterThan,
            ">=" => OperatorComparison::GeneralGreaterOrEquals,

            "eq" => OperatorComparison::ValueEquals,
            "ne" => OperatorComparison::ValueNotEquals,
            "lt" => OperatorComparison::ValueLessThan,
            "le" => OperatorComparison::ValueLessOrEquals,
            "gt" => OperatorComparison::ValueGreaterThan,
            "ge" => OperatorComparison::ValueGreaterOrEquals,

            _ => panic!("internal error"),
        };

        found_expr(
            input,
            Expr::Comparison { left: Box::new(left), operator, right: Box::new(right) }
        )
    } else {
        Ok((current_input, left))
    }
}

// [86]    	StringConcatExpr 	   ::=    	RangeExpr ( "||" RangeExpr )*
parse_sequence!(parse_string_concat_expr, "||", parse_range_expr, StringConcat);

// [87]    	RangeExpr 	   ::=    	AdditiveExpr ( "to" AdditiveExpr )?
fn parse_range_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, from) = parse_additive_expr(input)?;

    let check = ws1_tag_ws1("to", input);
    if check.is_ok() {
        let input = check?.0;

        let (input, till) = parse_additive_expr(input)?;

        found_expr(
            input,
            Expr::Range { from: Box::new(from), till: Box::new(till) }
        )
    } else {
        Ok((input, from))
    }
}

// [88]    	AdditiveExpr 	   ::=    	MultiplicativeExpr ( ("+" | "-") MultiplicativeExpr )*
fn parse_additive_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, operand) = parse_multiplicative_expr(input)?;

    let mut left = operand;

    let mut current_input = input;
    loop {
        let check = alt((
            preceded(ws, tag("+")),
            preceded(ws1, tag("-"))
        ))(current_input);
        if check.is_ok() {
            let (input, sign) = check?;

            let (input, right) = parse_multiplicative_expr(input)?;
            current_input = input;

            let operator = match sign {
                "+" => OperatorArithmetic::Plus,
                "-" => OperatorArithmetic::Minus,
                _ => panic!("internal error")
            };

            left = Expr::Binary { left: Box::new(left), operator, right: Box::new(right)}
        } else {
            break
        }
    }
    Ok((current_input, left))
}

// [89]    	MultiplicativeExpr 	   ::=    	UnionExpr ( ("*" | "div" | "idiv" | "mod") UnionExpr )*
fn parse_multiplicative_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, operand) = parse_union_expr(input)?;

    let mut left = operand;

    let mut current_input = input;
    loop {
        let check = alt((
            delimited(ws,tag("*"), ws),
            delimited(ws1,alt((tag("div"), tag("idiv"), tag("mod"))), ws1)
        ))(current_input);
        if check.is_ok() {
            let (input, sign) = check?;

            let (input, right) = parse_union_expr(input)?;
            current_input = input;

            let operator = match sign {
                "*" => OperatorArithmetic::Multiply,
                "div" => OperatorArithmetic::Divide,
                "idiv" => OperatorArithmetic::IDivide,
                "mod" => OperatorArithmetic::Mod,
                _ => panic!("internal error")
            };

            left = Expr::Binary { left: Box::new(left), operator, right: Box::new(right) }
        } else {
            break
        }
    }
    Ok((current_input, left))
}

// [90]    	UnionExpr 	   ::=    	IntersectExceptExpr ( ("union" | "|") IntersectExceptExpr )*
fn parse_union_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_intersect_except_expr(input)?;

    let mut exprs = vec![];
    exprs.push(expr);

    let mut current_input = input;
    loop {
        let check = preceded(ws1, alt((tag("union"), tag("|"))))(current_input);
        if check.is_err() {
            break
        } else {
            let (input, _) = check?;
            let (input, expr) = parse_intersect_except_expr(input)?;
            current_input = input;

            exprs.push(expr);
        }
    }

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((current_input, expr))
    } else {
        found_expr(current_input, Expr::Union(exprs))
    }
}

// [91]    	IntersectExceptExpr 	   ::=    	InstanceofExpr ( ("intersect" | "except") InstanceofExpr )*
fn parse_intersect_except_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_instanceof_expr(input)?;

    let mut left = expr;

    let mut current_input = input;
    loop {
        let check = preceded(ws1, alt((tag("intersect"), tag("except"))))(current_input);
        if check.is_err() {
            break
        } else {
            let (input, op) = check?;
            let (input, right) = parse_instanceof_expr(input)?;
            current_input = input;

            let is_intersect = match op {
                "intersect" => true,
                "except" => false,
                _ => panic!("internal error")
            };

            left = Expr::IntersectExcept { left: Box::new(left), is_intersect, right: Box::new(right) }
        }
    }

    Ok((current_input, left))
}

// [92]    	InstanceofExpr 	   ::=    	TreatExpr ( "instance" "of" SequenceType )?
fn parse_instanceof_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_treat_expr(input)?;

    let check = ws1_tag_ws1("instance", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("of", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Expr::Treat { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [93]    	TreatExpr 	   ::=    	CastableExpr ( "treat" "as" SequenceType )?
fn parse_treat_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_castable_expr(input)?;

    let check = ws1_tag_ws1("treat", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Expr::Castable { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [94]    	CastableExpr 	   ::=    	CastExpr ( "castable" "as" SingleType )?
fn parse_castable_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_cast_expr(input)?;

    let check = ws1_tag_ws1("castable", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Expr::Castable { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [95]    	CastExpr 	   ::=    	ArrowExpr ( "cast" "as" SingleType )?
fn parse_cast_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_arrow_expr(input)?;

    let check = ws1_tag_ws1("cast", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Expr::Castable { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [96]    	ArrowExpr 	   ::=    	UnaryExpr ( "=>" ArrowFunctionSpecifier ArgumentList )*
// [127]    	ArrowFunctionSpecifier 	   ::=    	EQName | VarRef | ParenthesizedExpr
fn parse_arrow_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_unary_expr(input)?;

    let check = ws1_tag_ws1("=>", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;

        // let (input, st) = parse_arrow_function_specifier(input);

        todo!()
    }
}

// [97]    	UnaryExpr 	   ::=    	("-" | "+")* TODO: ValueExpr
// [98]    	ValueExpr 	   ::=    	TODO: ValidateExpr | TODO: ExtensionExpr | SimpleMapExpr
fn parse_unary_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let mut is_positive: Option<bool> = None;
    let mut current_input = input;

    //TODO: optimize by relaxing
    loop {
        let input = ws(current_input)?.0;

        let check = one_of("-+")(input);
        if check.is_ok() {
            let (input, op) = check?;
            current_input = input;

            if op == '+' {
                is_positive = Some(is_positive.unwrap_or(true));
            } else {
                is_positive = Some(!is_positive.unwrap_or(true));
            }
        } else {
            break;
        }
    }

    let (input, expr) = parse_simple_map_expr(current_input)?;
    if let Some(sign_is_positive) = is_positive {
        found_expr(input, Expr::Unary { expr: Box::new(expr), sign_is_positive })
    } else {
        Ok((input, expr))
    }
}

// [107]    	SimpleMapExpr 	   ::=    	PathExpr ("!" PathExpr)*
parse_sequence!(parse_simple_map_expr, "!", parse_path_expr, SimpleMap);

// [108]    	PathExpr 	   ::=    	("/" RelativePathExpr?) | ("//" RelativePathExpr) | RelativePathExpr
fn parse_path_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let check = alt((tag("//"), tag("/")))(input);
    if check.is_ok() {
        let (input, steps) = check?;
        let check = parse_relative_path_expr(input);
        if check.is_ok() {
            let (input, expr) = check?;
            return found_expr(input, Expr::InitialPath { steps: op::Steps::from(steps), expr: Box::new(expr) })
        } else {
            if steps == "/" {
                return found_expr(input, Expr::Root)
            }
        }
    }

    parse_relative_path_expr(input)
}

// [109]    	RelativePathExpr 	   ::=    	StepExpr (("/" | "//") StepExpr)*
fn parse_relative_path_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let mut exprs = vec![];

    let (input, expr) = parse_step_expr(input)?;
    let mut current_input = input;

    exprs.push(expr);

    loop {
        let check = alt((tag("//"), tag("/")) )(current_input);
        if check.is_ok() {
            let (input, steps) = check?;
            current_input = input;

            let (input, expr) = parse_step_expr(current_input)?;
            current_input = input;

            exprs.push(Expr::Path { steps: op::Steps::from(steps), expr: Box::new(expr) })
        } else {
            break
        }
    }

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((current_input, expr))
    } else {
        found_expr(current_input, Expr::Steps(exprs))
    }
}

// [110]    	StepExpr 	   ::=    	PostfixExpr | AxisStep
parse_one_of!(parse_step_expr, Expr,
    parse_postfix_expr, parse_axis_step,
);

// [111]    	AxisStep 	   ::=    	(ReverseStep | ForwardStep) PredicateList
// [123]    	PredicateList 	   ::=    	Predicate*
fn parse_axis_step(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    // TODO let check = parse_reverse_step(input);

    let (input, step) = parse_forward_step(input)?;

    let (input, predicates) = parse_predicate_list(input)?;

    found_expr(input, Expr::AxisStep { step: Box::new(step), predicates } )

}

// [112]    	ForwardStep 	   ::=    	TODO (ForwardAxis NodeTest) | AbbrevForwardStep
fn parse_forward_step(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_abbrev_forward_step(input)
}

// [114]    	AbbrevForwardStep 	   ::=    	"@"? NodeTest
fn parse_abbrev_forward_step(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let check = tag("@")(input);
    let (input, attribute) = if check.is_ok() {
        let (input, _) = check?;
        (input, true)
    } else {
        (input, false)
    };

    let (input, test) = parse_node_test(input)?;

    found_expr(input, Expr::ForwardStep { attribute, test: Box::new(test) } )
}

// [115]    	ReverseStep 	   ::=    	(ReverseAxis NodeTest) | AbbrevReverseStep

// [118]    	NodeTest 	   ::=    	KindTest | NameTest
// TODO: parse_one_of!(parse_node_test, parse_kind_test, parse_name_test);
fn parse_node_test(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_name_test(input)
}

// [119]    	NameTest 	   ::=    	EQName | Wildcard
// [120]    	Wildcard 	   ::=    	"*"
// | (NCName ":*")
// | ("*:" NCName)
// TODO: | (BracedURILiteral "*") 	/* ws: explicit */
fn parse_name_test(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let check = parse_eqname(input);
    let (input, qname) = if check.is_ok() {
        let (input, name) = check?;
        (input, name)
    } else {
        let check = tag("*:")(input);
        if check.is_ok() {
            let (input, _) = check?;
            let (input, name) = parse_ncname(input)?;
            (input, QName::new("*".to_string(), name))
        } else {
            let check = tag("*")(input);
            if check.is_ok() {
                let (input, _) = check?;
                (input, QName::new("*".to_string(), "*".to_string()))
            } else {
                let (input, prefix) = parse_ncname(input)?;
                let (input, _) = tag(":*")(input)?;

                (input, QName::new(prefix, "*".to_string()))
            }
        }
    };

    found_expr(input, Expr::NameTest(qname))
}

// [121]    	PostfixExpr 	   ::=    	PrimaryExpr (Predicate | TODO: ArgumentList | Lookup)*
fn parse_postfix_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("parse_postfix_expr {:?}", input);
    }

    let (input, _) = ws(input)?;
    let (input, primary) = parse_primary_expr(input)?;

    let mut suffix = Vec::new();

    let mut current_input = input;

    loop {
        let check = parse_predicate(current_input);
        if check.is_ok() {
            let (input, predicate) = check?;
            current_input = input;

            suffix.push(predicate)
        } else {
            break;
        }
    }

    if suffix.len() == 0 {
        Ok((current_input, primary))
    } else {
        found_expr(
            current_input,
            Expr::Postfix { primary: Box::new(primary), suffix }
        )
    }
}

// [122]    	ArgumentList 	   ::=    	"(" (Argument ("," Argument)*)? ")"
fn parse_argument_list(input: &str) -> IResult<&str, Vec<Expr>, CustomError<&str>> {
    let (input, _) = ws_tag("(", input)?;

    let (input, arguments) = parse_arguments(input)?;

    let (input, _) = ws_tag(")", input)?;

    found_exprs(
        input,
        arguments
    )
}

// [123]    	PredicateList 	   ::=    	Predicate*
fn parse_predicate_list(input: &str) -> IResult<&str, Vec<Expr>, CustomError<&str>> {
    let mut current_input = input;

    let mut predicates = vec![];

    loop {
        let check = parse_predicate(current_input);
        if check.is_err() {
            break
        }
        let (input, predicate) = check?;
        current_input = input;

        predicates.push(predicate);
    }

    found_exprs(current_input, predicates)
}

// [124]    	Predicate 	   ::=    	"[" Expr "]"
fn parse_predicate(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("parse_predicate: {:?}", input);
    }

    let input = ws_tag("[", input)?.0;

    let (input, expr) = parse_expr_single(input)?;
//    let (input, expr) = parse_expr(input)?;

    let input = ws_tag("]", input)?.0;

    Ok((input, Expr::Predicate(Box::new(expr))))
}

// [127]    	ArrowFunctionSpecifier 	   ::=    	TODO: EQName | VarRef | ParenthesizedExpr
// parse_one_of!(
//     parse_arrow_function_specifier,
//     parse_eqname, parse_var_ref, parse_parenthesized_expr,
// );

// [128]    	PrimaryExpr 	   ::=    	Literal
//  | VarRef
//  | ParenthesizedExpr
//  | ContextItemExpr
//  | FunctionCall
//  TODO: | OrderedExpr
//  TODO: | UnorderedExpr
//  | NodeConstructor
//  | FunctionItemExpr
//  | MapConstructor
//  | ArrayConstructor
//  TODO: | StringConstructor
//  TODO: | UnaryLookup
parse_one_of!(parse_primary_expr, Expr,
    parse_literal,
    parse_var_ref,
    parse_parenthesized_expr,
    parse_context_item_expr,
    parse_function_call,
    parse_node_constructor,
    parse_function_item_expr,
    parse_map_constructor,
    parse_array_constructor,
    //
);

// [131]    	VarRef 	   ::=    	"$" VarName
// [132]    	VarName 	   ::=    	EQName
fn parse_var_ref(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    Ok((
        input,
        Expr::VarRef { name }
    ))
}

// [133]    	ParenthesizedExpr 	   ::=    	"(" Expr? ")"
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let (input, _) = ws_tag("(", input)?;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        let (input, result) = check?;
        (input, Expr::Sequence(Box::new(result)))
    } else {
        (input, Expr::SequenceEmpty())
    };

    let (input, _) = ws_tag(")", input)?;

    Ok((input, expr))
}

// [134]    	ContextItemExpr 	   ::=    	"."
fn parse_context_item_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag(".", input)?;

    Ok((
        input,
        Expr::ContextItem
    ))
}

// [137]    	FunctionCall 	   ::=    	EQName ArgumentList
fn parse_function_call(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, function) = parse_eqname(input)?;
    let (input, arguments) = parse_argument_list(input)?;

    //workaround: lookahead for inline function
    let check = ws_tag("{", input);
    if check.is_ok() {
        // TODO: is it correct?
        Err(nom::Err::Error(nom::error::ParseError::from_char(input, ' ')))
    } else {
        Ok((
            input,
            Expr::Call { function, arguments }
        ))
    }
}

// [138]    	Argument 	   ::=    	ExprSingle TODO: | ArgumentPlaceholder
// TODO: (Argument ("," Argument)*)?
fn parse_arguments(input: &str) -> IResult<&str, Vec<Expr>, CustomError<&str>> {
    let mut arguments = vec![];

    let mut current_input = input;

    let check = parse_expr_single(current_input);
    match check {
        Ok((input, argument)) => {
            current_input = input;

            arguments.push(argument);

            loop {
                let tmp = ws_tag(",", current_input);
                if tmp.is_err() {
                    return
                        found_exprs(
                            current_input,
                            arguments
                        )
                }
                current_input = tmp?.0;

                let (input, argument) = parse_expr_single(current_input)?;
                current_input = input;

                arguments.push(argument);
            }
        },
        Err(nom::Err::Failure(code)) => Err(nom::Err::Failure(code)),
        _ => {
            found_exprs(current_input, arguments)
        }
    }
}

// [167]    	FunctionItemExpr 	   ::=    	NamedFunctionRef | InlineFunctionExpr
parse_one_of!(parse_function_item_expr, Expr,
    parse_named_function_ref,
    parse_inline_function_expr,
);

// [168]    	NamedFunctionRef 	   ::=    	EQName "#" IntegerLiteral
fn parse_named_function_ref(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("parse_named_function_ref {:?}", input);
    }

    let (input, _) = ws(input)?;
    let (input, name) = parse_eqname(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, number) = parse_integer_literal(input)?;

    Ok((
        input,
        Expr::NamedFunctionRef { name, arity: Box::new(number) }
    ))
}

// [169]    	InlineFunctionExpr 	   ::=    	Annotation* "function" "(" ParamList? ")" ("as" SequenceType)? FunctionBody
// [35]    	FunctionBody 	   ::=    	EnclosedExpr
fn parse_inline_function_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    // TODO: Annotation*

    let (input, _) = ws_tag("function", input)?;
    let (input, _) = ws_tag("(", input)?;

    let check = parse_param_list(input);
    let (input, arguments) = if check.is_ok() {
        let (input, expr) = check?;

        let params = expr_to_params(expr);

        // TODO: Expr to vec![Param]
        (input, params)
    } else {
        (input, vec![])
    };

    let (input, _) = ws_tag(")", input)?;

    // TODO: ("as" SequenceType)?

    let (input, body) = parse_enclosed_expr(input)?;

    Ok((
        input,
        Expr::Function { arguments, body: Box::new(body) }
    ))
}

// [170]    	MapConstructor 	   ::=    	"map" "{" (MapConstructorEntry ("," MapConstructorEntry)*)? "}"
fn parse_map_constructor(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let input = ws_tag("map", input)?.0;

    let input = ws_tag("{", input)?.0;

    let mut entries = vec![];

    let mut current_input = input;
    loop {
        if DEBUG {
            println!("parse_map_entries: {:?}", current_input);
        }

        let result = parse_map_constructor_entry(current_input);
        if result.is_err() {

            current_input = tag("}")(input)?.0;

            return
                Ok((
                    current_input,
                    Expr::Map { entries }
                ))
        }
        let (input, entry) = result?;

        entries.push(entry);

        let input = ws(input)?.0;

        let tmp = tag(",")(input);
        if tmp.is_err() {

            current_input = tag("}")(input)?.0;

            return
                Ok((
                    current_input,
                    Expr::Map { entries }
                ))
        }
        current_input = tmp?.0;
    }
}

// [171]    	MapConstructorEntry 	   ::=    	MapKeyExpr ":" MapValueExpr
// [172]    	MapKeyExpr 	   ::=    	ExprSingle
// [173]    	MapValueExpr 	   ::=    	ExprSingle
fn parse_map_constructor_entry(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, key) = parse_expr_single(input)?;

    let input = ws_tag(":", input)?.0;

    let (input, value) = parse_expr_single(input)?;

    Ok((
        input,
        Expr::MapEntry { key: Box::new( key ), value: Box::new( value ) }
    ))
}

// [174]    	ArrayConstructor 	   ::=    	SquareArrayConstructor | CurlyArrayConstructor
parse_one_of!(parse_array_constructor, Expr,
    parse_square_array_constructor, parse_curly_array_constructor,
);

// [175]    	SquareArrayConstructor 	   ::=    	"[" (ExprSingle ("," ExprSingle)*)? "]"
fn parse_square_array_constructor(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("[", input)?;

    let mut exprs = vec![];

    let mut current_input = input;

    let check = parse_expr_single(current_input);
    if check.is_ok() {
        let (input, expr) = check?;
        current_input = input;

        exprs.push(expr);

        loop {
            let check = ws_tag(",", current_input);
            if check.is_ok() {
                let (input, _) = check?;
                current_input = input;

                let (input, expr) = parse_expr_single(current_input)?;
                current_input = input;

                exprs.push(expr);
            } else {
                break
            }
        }
    }
    let (input, _) = ws_tag("]", current_input)?;

    found_expr(input, Expr::SquareArrayConstructor(exprs))
}

// [176]    	CurlyArrayConstructor 	   ::=    	"array" EnclosedExpr
fn parse_curly_array_constructor(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("array", input)?;

    let (input, expr) = parse_enclosed_expr(input)?;

    found_expr(input, Expr::CurlyArrayConstructor(Box::new(expr)))
}

// [182]    	SingleType 	   ::=    	SimpleTypeName "?"?
// [205]    	SimpleTypeName 	   ::=    	TypeName
// [206]    	TypeName 	   ::=    	EQName
fn parse_single_type(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, name) = parse_eqname(input)?;

    let check = tag("?")(input);
    if check.is_ok() {
        let (input, _) = check?;

        todo!()
    }
    todo!()
}

// [183]    	TypeDeclaration 	   ::=    	"as" SequenceType
fn parse_type_declaration(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("as", input)?;

    parse_sequence_type(input)
}

// [184]    	SequenceType 	   ::=    	("empty-sequence" "(" ")")
// | (ItemType OccurrenceIndicator?)
// [185]    	OccurrenceIndicator 	   ::=    	"?" | "*" | "+"
fn parse_sequence_type(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let check = tag("empty-sequence")(input);
    if check.is_ok() {
        let input = check?.0;

        let input = ws_tag("(", input)?.0;
        let input = ws_tag(")", input)?.0;

        Ok((
            input,
            Expr::SequenceEmpty()
        ))
    } else {
        let (input, item_type) = parse_item_type(input)?;

        let check: Result<(&str, &str), nom::Err<Error<&str>>> = alt((tag("?"), tag("*"), tag("+")))(input);
        let (input, occurrence_indicator) = if check.is_ok() {
            let (input, sign) = check.unwrap();
            let oi = match sign {
                "?" => OccurrenceIndicator::ZeroOrOne,
                "*" => OccurrenceIndicator::ZeroOrMore,
                "+" => OccurrenceIndicator::OneOrMore,
                _ => panic!("internal error")
            };
            (input, oi)
        } else {
            (input, OccurrenceIndicator::ExactlyOne)
        };

        Ok((
            input,
            Expr::SequenceType { item_type, occurrence_indicator }
        ))
    }
}

// TODO [186]    	ItemType 	   ::=    	KindTest | ("item" "(" ")") | FunctionTest | MapTest | ArrayTest | AtomicOrUnionType | ParenthesizedItemType
parse_one_of!(parse_item_type, ItemType,
    parse_item, parse_atomic_or_union_type,
);

fn parse_item(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    let (input, _) = ws_tag("item", input)?;
    let (input, _) = ws_tag("(", input)?;
    let (input, _) = ws_tag(")", input)?;

    Ok((input, ItemType::Item))
}

// [187]    	AtomicOrUnionType 	   ::=    	EQName
fn parse_atomic_or_union_type(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    let (input, name) = parse_eqname(input)?;

    Ok((input, ItemType::AtomicOrUnionType(name)))
}

// [226]    	EscapeQuot 	   ::=    	'""'
fn parse_escape_quot(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = tag("\"\"")(input)?;

    Ok((input, Expr::EscapeQuot))
}

// [227]    	EscapeApos 	   ::=    	"''"
fn parse_escape_apos(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = tag("''")(input)?;

    Ok((input, Expr::EscapeApos))
}

fn expr_to_qname(expr: Expr) -> QName {
    match expr {
        Expr::QName { prefix, url, local_part } => QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", expr)
    }
}