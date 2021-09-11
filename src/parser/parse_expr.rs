use crate::parse_one_of;
use crate::parse_sequence;
use crate::parse_surroundings;
use crate::parser::op;
use crate::parser::errors::CustomError;

use nom::{branch::alt, bytes::complete::{is_not, tag, take_till, take_until, take_while, take_while1, take_while_m_n}, character::complete::{multispace0, multispace1, one_of}, error::Error, IResult, InputTakeAtPosition};

use crate::parser::helper::*;
use crate::fns::expr_to_params;
use crate::value::QName;
use crate::parser::parse_literal::{parse_literal, parse_integer_literal};
use crate::parser::parse_xml::parse_node_constructor;
use crate::parser::parse_names::{parse_eqname, parse_ncname};
use crate::parser::op::{Expr, found_expr, Operator, Statement, found_exprs, found_statements, ItemType, OccurrenceIndicator};

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
            current_input = input;

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
    // TODO: TypeDeclaration?

    found_expr(
        input,
        Expr::Param { name, type_declaration: Box::new(None)}
    )
}

// [32]    	FunctionDecl 	   ::=    	"function" EQName "(" ParamList? ")" ("as" SequenceType)? (FunctionBody | "external")
// [35]    	FunctionBody 	   ::=    	EnclosedExpr
fn parse_function_decl(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag_ws("function", input)?;

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

    let (input, _) = ws_tag_ws(")", current_input)?;
    current_input = input;

    let check = parse_type_declaration(current_input);
    let type_declaration = if check.is_ok() {
        let (input, td) = check?;
        current_input = input;

        Box::new(Some(td))
    } else {
        Box::new(None)
    };

    let check = ws_tag_ws("external", current_input);
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
//  TODO: | IfExpr
//  TODO: | TryCatchExpr
// | OrExpr
parse_one_of!(
    parse_expr_single, Expr,
    parse_flwor_expr,
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

    let (input, _) = ws_tag_ws("return", current_input)?;
    current_input = input;

    let (input, return_expr) = parse_expr_single(current_input)?;
    current_input = input;

    found_expr(
        current_input,
        Expr::FLWOR { clauses, return_expr: Box::new(return_expr) }
    )
}

// [42]    	InitialClause 	   ::=    	ForClause | LetClause | WindowClause
fn parse_initial_clause(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_let_clause_expr(input)
}

// [43]    	IntermediateClause 	   ::=    	InitialClause | WhereClause | GroupByClause | OrderByClause | CountClause
fn parse_intermediate_clause(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_let_clause_expr(input)
}

// [48]    	LetClause 	   ::=    	"let" LetBinding ("," LetBinding)*
fn parse_let_clause_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("parse_let_clause_expr {:?}", input);
    }

    let check = ws_tag("let", input);
    if check.is_ok() {
        let input = check?.0;

        let mut bindings = vec![];

        let mut current_input = input;
        loop {
            let (input, expr) = parse_let_binding_expr(current_input)?;

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
    } else {
        // TODO: is it correct?
        Err(nom::Err::Error(nom::error::ParseError::from_char(input, ' ')))
    }
}

// [49]    	LetBinding 	   ::=    	"$" VarName TypeDeclaration? ":=" ExprSingle
// [132]    	VarName 	   ::=    	EQName
fn parse_let_binding_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

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

// [83]    	OrExpr 	   ::=    	AndExpr ( "or" AndExpr )*
parse_sequence!(parse_or_expr, "or", parse_and_expr, Or);

// [84]    	AndExpr 	   ::=    	ComparisonExpr ( "and" ComparisonExpr )*
parse_sequence!(parse_and_expr, "and", parse_comparison_expr, And);

// [85]    	ComparisonExpr 	   ::=    	StringConcatExpr ( ( TODO ValueComp
// TODO | GeneralComp
// TODO | NodeComp) StringConcatExpr )?
fn parse_comparison_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    parse_string_concat_expr(input)
}

// [86]    	StringConcatExpr 	   ::=    	RangeExpr ( "||" RangeExpr )*
parse_sequence!(parse_string_concat_expr, "||", parse_range_expr, StringConcat);

// [87]    	RangeExpr 	   ::=    	AdditiveExpr ( "to" AdditiveExpr )?
fn parse_range_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, from) = parse_binary_expr(input)?;

    let check = ws_tag("to", input);
    if check.is_ok() {
        let input = check?.0;

        let (input, till) = parse_binary_expr(input)?;

        found_expr(
            input,
            Expr::Range { from: Box::new(from), till: Box::new(till) }
        )
    } else {
        found_expr(input, from)
    }
}

// [85]    	ComparisonExpr 	   ::=    	 TODO: StringConcatExpr ( (ValueComp
// TODO: | GeneralComp
// TODO: | NodeComp) StringConcatExpr )?
// [88]    	AdditiveExpr 	   ::=    	MultiplicativeExpr ( ("+" | "-") MultiplicativeExpr )*
// [89]    	MultiplicativeExpr 	   ::=    	UnionExpr ( ("*" | "div" | "idiv" | "mod") UnionExpr )*
// [99] GeneralComp    ::=    	"=" | "!=" | "<" | "<=" | ">" | ">="
// [100] ValueComp 	   ::=    	"eq" | "ne" | "lt" | "le" | "gt" | "ge"
// [101] NodeComp 	   ::=    	"is" | "<<" | ">>"
fn parse_binary_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    if DEBUG {
        println!("parse_binary_expr 1: {:?}", input);
    }

    let (input, expr) = parse_union_expr(input)?;

    let mut left = expr;
    let mut current_input = input;

    loop {
        if DEBUG {
            println!("parse_binary_expr 2: {:?}", current_input);
        }

        current_input = ws(current_input)?.0;

        let check = alt(
            (
                tag("+"), tag("-"), tag("*"), tag("div"), tag("idiv"), tag("mod"),
                tag("="), tag("!="), tag("<"), tag("<="), tag(">"), tag(">="),
                tag("eq"), tag("ne"), tag("lt"), tag("le"), tag("gt"), tag("ge"),
                // TODO: tag("is"), tag("<<"), tag(">>"),
            )
        )(current_input);

        if check.is_ok() {
            let (mut input, op) = check?;

            input = ws(input)?.0;

            let (input, right) = parse_union_expr(input)?;
            current_input = input;

            if DEBUG {
                println!("parse_binary_expr: {:?} {:?} {:?}", left, op, right);
            }

            let operator = match op {
                "=" => Operator::GeneralEquals,
                "!=" => Operator::GeneralNotEquals,
                "<" => Operator::GeneralLessThan,
                "<=" => Operator::GeneralLessOrEquals,
                ">" => Operator::GeneralGreaterThan,
                ">=" => Operator::GeneralGreaterOrEquals,

                "eq" => Operator::ValueEquals,
                "ne" => Operator::ValueNotEquals,
                "lt" => Operator::ValueLessThan,
                "le" => Operator::ValueLessOrEquals,
                "gt" => Operator::ValueGreaterThan,
                "ge" => Operator::ValueGreaterOrEquals,

                _ => Operator::Unknown,
            };

            if operator != Operator::Unknown {
                left = Expr::Comparison { left: Box::new(left), operator, right: Box::new(right) };
            } else {
                let operator = match op {
                    "+" => Operator::Plus,
                    "-" => Operator::Minus,
                    "*" => Operator::Multiply,
                    "div" => Operator::Divide,
                    "idiv" => Operator::IDivide,
                    "mod" => Operator::Mod,
                    _ => panic!("this must not happen") // TODO: raise error instead
                };

                left = Expr::Binary { left: Box::new(left), operator, right: Box::new(right) }
            }
        } else {
            return found_expr(
                current_input,
                left
            );
        }
    }
}

// [90]    	UnionExpr 	   ::=    	IntersectExceptExpr TODO: ( ("union" | "|") IntersectExceptExpr )*
fn parse_union_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_intersect_except_expr(input)?;

    let check: Result<(&str, &str), nom::Err<Error<&str>>> = alt((tag("union"), tag("|")))(input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        todo!()
    }
}

// [91]    	IntersectExceptExpr 	   ::=    	InstanceofExpr TODO: ( ("intersect" | "except") InstanceofExpr )*
fn parse_intersect_except_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_instanceof_expr(input)?;

    let check: Result<(&str, &str), nom::Err<Error<&str>>> = alt((tag("intersect"), tag("except")))(input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        todo!()
    }
}

// [92]    	InstanceofExpr 	   ::=    	TreatExpr ( "instance" "of" SequenceType )?
fn parse_instanceof_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_treat_expr(input)?;

    let check = ws_tag_ws("instance", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = ws_tag_ws("of", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Expr::TreatExpr { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [93]    	TreatExpr 	   ::=    	CastableExpr ( "treat" "as" SequenceType )?
fn parse_treat_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_castable_expr(input)?;

    let check = ws_tag_ws("treat", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = ws_tag_ws("as", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Expr::CastableExpr { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [94]    	CastableExpr 	   ::=    	CastExpr ( "castable" "as" SingleType )?
fn parse_castable_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_cast_expr(input)?;

    let check = ws_tag_ws("castable", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = ws_tag_ws("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Expr::CastableExpr { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [95]    	CastExpr 	   ::=    	ArrowExpr ( "cast" "as" SingleType )?
fn parse_cast_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_arrow_expr(input)?;

    let check = ws_tag_ws("cast", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = ws_tag_ws("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Expr::CastableExpr { expr: Box::new(expr), st: Box::new(st) } )
    }
}

// [96]    	ArrowExpr 	   ::=    	UnaryExpr ( "=>" ArrowFunctionSpecifier ArgumentList )*
// [127]    	ArrowFunctionSpecifier 	   ::=    	EQName | VarRef | ParenthesizedExpr
fn parse_arrow_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, expr) = parse_unary_expr(input)?;

    let check = ws_tag_ws("=>", input);
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

    let mut is_positive = true;
    let mut current_input = input;

    //TODO: optimize by relaxing
    loop {
        let input = ws(current_input)?.0;

        let check = one_of("-+")(input);
        if check.is_ok() {
            let (input, op) = check?;
            current_input = input;

            if op == '-' {
                is_positive = !is_positive
            }
        } else {
            break;
        }
    }

    let check = parse_simple_map_expr(current_input);
    if check.is_ok() {
        let (input, expr) = check?;

        if is_positive {
            Ok((input, expr))
        } else {
            found_expr(input, Expr::Negative(Box::new(expr)))
        }
    } else {
        check
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
        found_expr(current_input, expr)
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
        found_expr(
            current_input,
            primary
        )
    } else {
        found_expr(
            current_input,
            Expr::Postfix { primary: Box::new(primary), suffix }
        )
    }
}

// [122]    	ArgumentList 	   ::=    	"(" (Argument ("," Argument)*)? ")"
fn parse_argument_list(input: &str) -> IResult<&str, Vec<Expr>, CustomError<&str>> {
    let (input, _) = ws_tag_ws("(", input)?;

    let (input, arguments) = parse_arguments(input)?;

    let (input, _) = ws_tag_ws(")", input)?;

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
    if check.is_ok() {
        let (input, argument) = check?;
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
    }
    found_exprs(
        current_input,
        arguments
    )
}

// [133]    	ParenthesizedExpr 	   ::=    	"(" Expr? ")"
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr, CustomError<&str>> {

    let input = ws_tag("(", input)?.0;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        let (input, result) = check?;
        (input, Expr::Sequence(Box::new(result)))
    } else {
        (input, Expr::SequenceEmpty())
    };

    let input = ws_tag(")", input)?.0;

    Ok((input, expr))
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

    Ok((
        input,
        Expr::SquareArrayConstructor { items: exprs }
    ))
}

// [176]    	CurlyArrayConstructor 	   ::=    	"array" EnclosedExpr
fn parse_curly_array_constructor(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let (input, _) = ws_tag("array", input)?;

    let (input, expr) = parse_enclosed_expr(input)?;

    Ok((
        input,
        Expr::CurlyArrayConstructor(Box::new(expr))
    ))
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
// | (ItemType TODO: OccurrenceIndicator?)
// TODO [185]    	OccurrenceIndicator 	   ::=    	"?" | "*" | "+"
fn parse_sequence_type(input: &str) -> IResult<&str, Expr, CustomError<&str>> {
    let check = ws_tag("empty-sequence", input);
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
        if check.is_ok() {
            todo!()
        }

        Ok((
            input,
            Expr::SequenceType { item_type, occurrence_indicator: OccurrenceIndicator::ExactlyOne }
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