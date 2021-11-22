use std::collections::{HashMap, HashSet};
use crate::parse_one_of;
use crate::parser::errors::{CustomError, IResultExt};
use crate::eval::prolog::*;

use nom::{branch::alt, bytes::complete::tag, error::Error, IResult};
use nom::bytes::complete::{is_a, is_not};
use nom::sequence::{preceded, delimited, tuple, terminated, separated_pair};
use nom::multi::{many0, many1, separated_list1};
use nom::combinator::{map, opt, peek};
use nom::character::complete::{one_of, digit1};
use crate::eval::{Axis, INS};

use crate::parser::helper::*;
use crate::fns::Param;
use crate::values::QName;
use crate::parser::parse_literal::{parse_literal, parse_integer_literal, parse_string_literal, parse_string_literal_as_string, parse_braced_uri_literal};
use crate::parser::parse_xml::parse_node_constructor;
use crate::parser::parse_names::{parse_eqname, parse_ncname, parse_ncname_expr};
use crate::parser::op::{found_expr, Statement, OperatorComparison, OperatorArithmetic};
use crate::eval::expression::{Expression, NodeTest};
use crate::eval::sequence_type::*;
use crate::eval::navigation::NodeParent;

// [2]    	VersionDecl 	   ::=    	"xquery" (("encoding" StringLiteral) | ("version" StringLiteral ("encoding" StringLiteral)?)) Separator
pub fn parse_version_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, (name, value)) = preceded(
        tuple((ws, tag("xquery"), ws1)),
        tuple((alt((tag("encoding"), tag("version"))), parse_string_literal_as_string))
    )(input)?;

    match name {
        "encoding" => {
            let (input, _) = tuple(
                (ws, tag(";"))
            )(input).or_failure(CustomError::XPST0003)?;

            match VersionDecl::boxed(Some(value), None) {
                Ok(expr) => Ok((input, expr)),
                Err((code, msg)) => Err(nom::Err::Failure(code))
            }
        },
        "version" => {
            let (input, encoding) = terminated(
                opt(preceded(
                    tuple((ws, tag("encoding"))), parse_string_literal_as_string
                )),
                tuple((ws, tag(";")))
            )(input)?;

            match VersionDecl::boxed(encoding, Some(value)) {
                Ok(expr) => Ok((input, expr)),
                Err((code, msg)) => Err(nom::Err::Failure(code))
            }
        },
        _ => panic!("internal error")
    }
}

// [3]    	MainModule 	   ::=    	Prolog QueryBody
pub(crate) fn parse_main_module(input: &str) -> IResult<&str, Vec<Statement>, CustomError<&str>> {
    map(
        tuple((
            preceded(ws, parse_prolog),
            preceded(ws, parse_expr)
        )),
        |(prolog, program)| vec![Statement::Prolog(prolog), Statement::Program(program)]
    )(input)
}

// [6]    	Prolog 	   ::=
// TODO: ((DefaultNamespaceDecl | Setter | NamespaceDecl | Import) Separator)*
// TODO: ((ContextItemDecl | AnnotatedDecl | OptionDecl) Separator)*
// [7]    	Separator 	   ::=    	";"
pub(crate) fn parse_prolog(input: &str) -> IResult<&str, Vec<Box<dyn Expression>>, CustomError<&str>> {

    let mut prolog = vec![];

    let mut current_input = input;
    loop {
        let check = terminated(
            alt((
                parse_default_namespace_decl, parse_setter, parse_namespace_decl
            )),
            tag(";")
        )(current_input);
        if check.is_ok() {
            let (input, expr) = check?;
            current_input = input;

            prolog.push(expr);
        } else {
            break
        }
    }

    loop {
        let check = terminated(
            alt((parse_annotated_decl, parse_option_decl)),
            tag(";")
        )(current_input);
        if check.is_ok() {
            let (input, expr) = check?;
            current_input = input;

            prolog.push(expr);
        } else {
            break
        }
    }

    Ok((current_input, prolog))
}

// [8]    	Setter 	   ::=    	BoundarySpaceDecl | DefaultCollationDecl | BaseURIDecl
// | ConstructionDecl | OrderingModeDecl | EmptyOrderDecl | CopyNamespacesDecl | DecimalFormatDecl
pub(crate) fn parse_setter(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    alt((
        parse_boundary_space_decl, parse_default_collation_decl, parse_base_uri_decl,
        parse_construction_decl, parse_ordering_mode_decl, parse_empty_order_decl,
        parse_copy_namespaces_decl, parse_decimal_format_decl
    ))(input)
}

// [9]    	BoundarySpaceDecl 	   ::=    	"declare" "boundary-space" ("preserve" | "strip")
pub(crate) fn parse_boundary_space_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("boundary-space"), ws1)),
            alt((tag("preserve"), tag("strip")))
        ),
        |mode| {
            let mode = match mode {
                "preserve" => BoundarySpace::Preserve,
                "strip" => BoundarySpace::Strip,
                _ => panic!("internal error")
            };
            DeclareBoundarySpace::boxed(mode)
        }
    )(input)
}

// [10]    	DefaultCollationDecl 	   ::=    	"declare" "default" "collation" URILiteral
pub(crate) fn parse_default_collation_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("default"), ws1, tag("collation"))),
            parse_uri_literal_as_string
        ),
        |uri| {
            DeclareDefaultCollation::boxed(uri)
        }
    )(input)
}

// [11]    	BaseURIDecl 	   ::=    	"declare" "base-uri" URILiteral
pub(crate) fn parse_base_uri_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("base-uri"), ws1)),
            parse_uri_literal
        ),
        |uri| DeclareBaseURI::boxed(uri)
    )(input)
}

// [12]    	ConstructionDecl 	   ::=    	"declare" "construction" ("strip" | "preserve")
pub(crate) fn parse_construction_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("construction"), ws1)),
            alt((tag("strip"), tag("preserve")))
        ),
        |mode| {
            let mode = match mode {
                "strip" => ConstructionMode::Strip,
                "preserve" => ConstructionMode::Preserve,
                _ => panic!("internal error")
            };
            DeclareConstruction::boxed(mode)
        }
    )(input)
}

// [13]    	OrderingModeDecl 	   ::=    	"declare" "ordering" ("ordered" | "unordered")
pub(crate) fn parse_ordering_mode_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("ordering"), ws1)),
            alt((tag("ordered"), tag("unordered")))
        ),
        |mode| {
            let mode = match mode {
                "ordered" => OrderingMode::Ordered,
                "unordered" => OrderingMode::Unordered,
                _ => panic!("internal error")
            };
            DeclareOrderingMode::boxed(mode)
        }
    )(input)
}

// [14]    	EmptyOrderDecl 	   ::=    	"declare" "default" "order" "empty" ("greatest" | "least")
pub(crate) fn parse_empty_order_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("default"), ws1, tag("order"), ws1, tag("empty"), ws1)),
            alt((tag("greatest"), tag("least")))
        ),
        |mode| {
            let mode = match mode {
                "greatest" => EmptyOrderMode::Greatest,
                "least" => EmptyOrderMode::Least,
                _ => panic!("internal error")
            };
            DeclareEmptyOrder::boxed(mode)
        }
    )(input)
}

// [15]    	CopyNamespacesDecl 	   ::=    	"declare" "copy-namespaces" PreserveMode "," InheritMode
// [16]    	PreserveMode 	   ::=    	"preserve" | "no-preserve"
// [17]    	InheritMode 	   ::=    	"inherit" | "no-inherit"
pub(crate) fn parse_copy_namespaces_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("copy-namespaces"), ws1)),
            tuple((
                alt((tag("preserve"), tag("no-preserve"))),
                ws, tag(","), ws,
                alt((tag("inherit"), tag("no-inherit"))),
            ))
        ),
        |(preserve_mode, _, _, _, inherit_mode)| {
            let preserve_mode = match preserve_mode {
                "preserve" => PreserveMode::Preserve,
                "no-preserve" => PreserveMode::NoPreserve,
                _ => panic!("internal error")
            };
            let inherit_mode = match inherit_mode {
                "inherit" => InheritMode::Inherit,
                "no-inherit" => InheritMode::NoInherit,
                _ => panic!("internal error")
            };
            DeclareCopyNamespaces::boxed(preserve_mode, inherit_mode)
        }
    )(input)
}

// [18]    	DecimalFormatDecl 	   ::=    	"declare" (("decimal-format" EQName) | ("default" "decimal-format")) (DFPropertyName "=" StringLiteral)*
// [19]    	DFPropertyName 	   ::=    	"decimal-separator" | "grouping-separator" | "infinity" | "minus-sign" | "NaN" | "percent" | "per-mille" | "zero-digit" | "digit" | "pattern-separator" | "exponent-separator"
pub(crate) fn parse_decimal_format_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        tuple((
            preceded(
                tuple((ws, tag("declare"), ws1, )),
                alt((
                    map(preceded(tag("decimal-format"), parse_eqname), |n| Some(n)),
                    map(preceded(tag("default"), tag("decimal-format")), |_| None)
                ))
            ),
            many0(parse_df_property)
        )),
        |(name, properties)| {
            let mut map = HashMap::with_capacity(properties.len());
            for (name, value) in properties {
                if map.insert(name, value) != None {
                    todo!()
                }
            }
            DeclareDecimalFormat::boxed(name, map)
        }
    )(input)
}

pub(crate) fn parse_df_property(input: &str) -> IResult<&str, (DecimalFormatPropertyName, String), CustomError<&str>> {
    map(
        preceded(
            ws,
            separated_pair(
                alt((
                    tag("decimal-separator"),
                    tag("grouping-separator"),
                    tag("infinity"),
                    tag("minus-sign"),
                    tag("NaN"),
                    tag("percent"),
                    tag("per-mille"),
                    tag("zero-digit"),
                    tag("digit"),
                    tag("pattern-separator"),
                    tag("exponent-separator")
                )),
                tuple((ws, tag("="), ws)),
                parse_string_literal_as_string
            )
        ),
        |(name, value)| {
            let name = match name {
                "decimal-separator" => DecimalFormatPropertyName::DecimalSeparator,
                "grouping-separator" => DecimalFormatPropertyName::GroupingSeparator,
                "infinity" => DecimalFormatPropertyName::Infinity,
                "minus-sign" => DecimalFormatPropertyName::MinusSign,
                "NaN" => DecimalFormatPropertyName::NaN,
                "percent" => DecimalFormatPropertyName::Percent,
                "per-mille" => DecimalFormatPropertyName::PerMille,
                "zero-digit" => DecimalFormatPropertyName::ZeroDigit,
                "digit" => DecimalFormatPropertyName::Digit,
                "pattern-separator" => DecimalFormatPropertyName::PatternSeparator,
                "exponent-separator" => DecimalFormatPropertyName::ExponentSeparator,
                _ => panic!("internal error")
            };
            (name, value)
        }
    )(input)
}

// [24]    	NamespaceDecl 	   ::=    	"declare" "namespace" NCName "=" URILiteral
pub(crate) fn parse_namespace_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("namespace"), ws1)),
            separated_pair(
                parse_ncname_expr,
                tuple((ws, tag("="), ws)),
                parse_uri_literal
            )
        ),
        |(name, uri)| DeclareNamespace::boxed(name, uri)
    )(input)
}

// [25]    	DefaultNamespaceDecl 	   ::=    	"declare" "default" ("element" | "function") "namespace" URILiteral
pub(crate) fn parse_default_namespace_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("default"), ws1)),
            separated_pair(
                alt((tag("element"), tag("function"))),
                tuple((ws1, tag("namespace"), ws1)),
                parse_uri_literal
            )
        ),
        |(name, uri)| DeclareDefaultNamespace::boxed(name, uri)
    )(input)
}


// [26]    	AnnotatedDecl 	   ::=    	"declare" Annotation* (VarDecl | FunctionDecl)
pub(crate) fn parse_annotated_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {

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
        check?
    } else {
        parse_function_decl(current_input)?
    };

    found_expr(input, Box::new(AnnotatedDecl { annotations, decl } ))
}

// [27]    	Annotation 	   ::=    	"%" EQName ("(" Literal ("," Literal)* ")")?
pub(crate) fn parse_annotation(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {

    let (input, _) = ws_tag("%", input)?;

    let (input, name) = parse_eqname(input)?;

    let check = parse_annotation_value(input);
    if check.is_ok() {
        let (input, list) = check?;
        todo!()
    } else {
        found_expr(input, Box::new(Annotation { name, value: None } ))
    }
}

pub(crate) fn parse_annotation_value(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, mut exprs) = delimited(
        tuple((ws, tag("("), ws)),
        separated_list1(
            tuple((ws, tag(","), ws)),
            parse_literal
        ),
        tuple((ws, tag(")"), ws))
    )(input)?;

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((input, expr))
    } else {
        Ok((input, Box::new(Literals { exprs })))
    }
}

// [28]    	VarDecl 	   ::=    	"variable" "$" VarName TypeDeclaration? ((":=" VarValue) | ("external" (":=" VarDefaultValue)?))
// [29]    	VarValue 	   ::=    	ExprSingle
// [30]    	VarDefaultValue 	   ::=    	ExprSingle
// [132]    	VarName 	   ::=    	EQName
pub(crate) fn parse_var_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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
        Box::new(VarDecl {
            external, name,
            type_declaration,
            value
        })
    )
}

// [33]    	ParamList 	   ::=    	Param ("," Param)*
fn parse_param_list(input: &str) -> IResult<&str, Vec<Param>, CustomError<&str>> {
    let (input, params) = separated_list1(
        tuple((ws, tag(","), ws)),
        parse_param
    )(input)?;

    let mut names = HashSet::with_capacity(params.len() as usize);
    for param in &params {
        if names.insert(param.name.clone()) == false {
            return Err(nom::Err::Failure(CustomError::XQST0039));
        }
    }

    Ok((input, params))
}

// [34]    	Param 	   ::=    	"$" EQName TypeDeclaration?
fn parse_param(input: &str) -> IResult<&str, Param, CustomError<&str>> {
    let (input, _) = tag("$")(input)?;
    let (input, name) = parse_eqname(input)?;

    let check = parse_type_declaration(input);
    let (input, sequence_type) = if check.is_ok() {
        let (input, expr) = check?;
        (input, Some(expr))
    } else {
        (input, None)
    };

    Ok((input, Param { name, sequence_type }))
}

// [32]    	FunctionDecl 	   ::=    	"function" EQName "(" ParamList? ")" ("as" SequenceType)? (FunctionBody | "external")
// [35]    	FunctionBody 	   ::=    	EnclosedExpr
fn parse_function_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws1_tag_ws1("function", input)?;

    let (input, name) = parse_function_name(input)?;

    let (input, _) = ws_tag_ws("(", input)?;
    let mut current_input = input;

    let params = match parse_param_list(input) {
        Ok((input, params)) => {
            current_input = input;
            params
        },
        Err(nom::Err::Failure(code)) => return Err(nom::Err::Failure(code)),
        Err(_) => vec![]
    };

    let (input, _) = ws_tag(")", current_input)?;
    current_input = input;

    let check = parse_type_declaration(current_input);
    let type_declaration = if check.is_ok() {
        let (input, td) = check?;
        current_input = input;

        Some(td)
    } else {
        None
    };

    let check = ws1_tag_ws1("external", current_input);
    let (input, external, body) = if check.is_ok() {
        let (input, _) = check?;

        (input, true, None)
    } else {
        let (input, body) = parse_enclosed_expr(current_input)?;
        (input, false, Some(body))
    };

    found_expr(input, Box::new(FunctionDecl { name, params, external, type_declaration, body } ))
}

// [36]    	EnclosedExpr 	   ::=    	"{" Expr? "}"
pub(crate) fn parse_enclosed_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws_tag_ws("{", input)?;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        check?
    } else {
        (input, Body::empty())
    };

    let (input, _) = ws_tag("}", input)?;

    Ok((input, EnclosedExpr::new(expr)))
}

// [37]    	OptionDecl 	   ::=    	"declare" "option" EQName StringLiteral
pub(crate) fn parse_option_decl(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("declare"), ws1, tag("option"), ws1)),
            tuple((parse_eqname, parse_string_literal_as_string))
        ),
        |(name,value)| DeclareOption::boxed(name, value)
    )(input)
}

// [38]    	QueryBody 	   ::=    	Expr
// [39]    	Expr 	   ::=    	ExprSingle ("," ExprSingle)*
pub(crate) fn parse_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let mut program = vec![];

    let mut current_input = input;
    loop {
        let (input, expr) = parse_expr_single(current_input)?;

        program.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                found_expr(input, Body::new(program))
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
parse_one_of!(parse_expr_single,
    parse_flwor_expr,
    parse_if_expr,
    parse_or_expr,
);

// [41]    	FLWORExpr 	   ::=    	InitialClause IntermediateClause* ReturnClause
// [69]    	ReturnClause 	   ::=    	"return" ExprSingle
fn parse_flwor_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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
        Box::new(FLWOR { clauses, return_expr })
    )
}

// [42]    	InitialClause 	   ::=    	ForClause | LetClause | TODO WindowClause
fn parse_initial_clause(input: &str) -> IResult<&str, Clause, CustomError<&str>> {
    alt((parse_for_clause, parse_let_clause))(input)
}

// [43]    	IntermediateClause 	   ::=    	InitialClause | TODO WhereClause | GroupByClause | OrderByClause | CountClause
fn parse_intermediate_clause(input: &str) -> IResult<&str, Clause, CustomError<&str>> {
    alt((
        parse_initial_clause,
        parse_where_clause
    ))(input)
}

// [44]    	ForClause 	   ::=    	"for" ForBinding ("," ForBinding)*
fn parse_for_clause(input: &str) -> IResult<&str, Clause, CustomError<&str>> {
    let (input, _) = ws_tag("for", input)?;

    let mut current_input = input;

    let mut bindings = vec![];
    loop {
        let (input, expr) = parse_for_binding(current_input)?;

        bindings.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return Ok((input, Clause::For(bindings)))
        }
        current_input = tmp?.0;
    }
}

// [45]    	ForBinding 	   ::=    	"$" VarName TypeDeclaration? AllowingEmpty? PositionalVar? "in" ExprSingle
// [46]    	AllowingEmpty 	   ::=    	"allowing" "empty"
// [47]    	PositionalVar 	   ::=    	"at" "$" VarName
fn parse_for_binding(input: &str) -> IResult<&str, Binding, CustomError<&str>> {
    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_var_name(input)?;

    let check = parse_type_declaration(input);
    let (input, st) = if check.is_ok() {
        let (input, st) = check?;
        (input, Some(st))
    } else {
        (input, None)
    };

    let check = tuple((ws1, tag("allowing"), ws1, tag("empty")))(input);
    let (input, allowing_empty) = if check.is_ok() {
        let (input, _) = check?;
        (input, true)
    } else {
        (input, false)
    };

    let check = preceded(
        tuple((ws1, tag("at"), ws1, tag("$"))),
        parse_var_name
    )(input);
    let (input, positional_var) = if check.is_ok() {
        let (input, name) = check?;
        (input, Some(name))
    } else {
        (input, None)
    };

    let (input, _) = ws_tag("in", input)?;

    let (input, values) = parse_expr_single(input)?;

    Ok((input, Binding::For { name, values, st, allowing_empty, positional_var }))
}

// [48]    	LetClause 	   ::=    	"let" LetBinding ("," LetBinding)*
fn parse_let_clause(input: &str) -> IResult<&str, Clause, CustomError<&str>> {
    let (input, _) = ws_tag("let", input)?;
    let mut current_input = input;

    let mut bindings = vec![];
    loop {
        let (input, expr) = parse_let_binding(current_input)?;

        bindings.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return Ok((input, Clause::Let(bindings)))
        }
        current_input = tmp?.0;
    }
}

// [49]    	LetBinding 	   ::=    	"$" VarName TypeDeclaration? ":=" ExprSingle
fn parse_let_binding(input: &str) -> IResult<&str, Binding, CustomError<&str>> {

    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_var_name(input)?;

    let check = parse_type_declaration(input);
    let (input, type_declaration) = if check.is_ok() {
        let (input, td) = check?;
        (input, Some(td))
    } else {
        (input, None)
    };

    let (input, _) = ws_tag(":=", input)?;

    let (input, value) = parse_expr_single(input)?;

    Ok((input, Binding::Let { name, st: type_declaration, value }))
}

// [60]    	WhereClause 	   ::=    	"where" ExprSingle
fn parse_where_clause(input: &str) -> IResult<&str, Clause, CustomError<&str>> {
    map(
        preceded(
            tuple((ws1, tag("where"))),
            parse_expr_single
        ),
        |expr| Clause::Where(expr)
    )(input)
}

// [77]    	IfExpr 	   ::=    	"if" "(" Expr ")" "then" ExprSingle "else" ExprSingle
fn parse_if_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws_tag("if", input)?;

    let (input, _) = ws_tag("(", input)?;

    let (input, condition) = parse_expr(input)?;

    let (input, _) = ws_tag(")", input)?;

    let (input, _) = ws_tag("then", input)?;

    let (input, consequence) = parse_expr_single(input)?;

    let (input, _) = ws_tag("else", input)?;

    let (input, alternative) = parse_expr_single(input)?;

    found_expr(input, Box::new(If { condition, consequence, alternative }))
}

// [83]    	OrExpr 	   ::=    	AndExpr ( "or" AndExpr )*
fn parse_or_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, mut exprs) = separated_list1(
        tuple((ws1, tag("or"), ws1)),
        parse_and_expr
    )(input)?;

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((input, expr))
    } else {
        Ok((input, Box::new(Or { exprs })))
    }
}

// [84]    	AndExpr 	   ::=    	ComparisonExpr ( "and" ComparisonExpr )*
fn parse_and_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, mut exprs) = separated_list1(
        tuple((ws1, tag("and"), ws1)),
        parse_comparison_expr
    )(input)?;

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((input, expr))
    } else {
        Ok((input, Box::new(And { exprs })))
    }
}

// [85]    	ComparisonExpr 	   ::=    	StringConcatExpr ( ( ValueComp
// | GeneralComp
// | NodeComp) StringConcatExpr )?
fn parse_comparison_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, left) = parse_string_concat_expr(input)?;

    let current_input = input;

    let check = delimited(
        ws1,
        alt((
            tag("is"), tag("<<"), tag(">>"),
            tag("="), tag("!="), tag("<="), tag("<"), tag(">="), tag(">"),
            tag("eq"), tag("ne"), tag("lt"), tag("le"), tag("gt"), tag("ge"),
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

            "is" => OperatorComparison::NodeIs,
            "<<" => OperatorComparison::NodePrecedes,
            ">>" => OperatorComparison::NodeFollows,

            _ => panic!("internal error"),
        };

        found_expr(input, Box::new(Comparison { left, operator, right }))
    } else {
        Ok((current_input, left))
    }
}

// [86]    	StringConcatExpr 	   ::=    	RangeExpr ( "||" RangeExpr )*
fn parse_string_concat_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, mut exprs) = separated_list1(
        tuple((ws1, tag("||"), ws1)),
        parse_range_expr
    )(input)?;

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((input, expr))
    } else {
        Ok((input, Box::new(StringConcat { exprs })))
    }
}

// [87]    	RangeExpr 	   ::=    	AdditiveExpr ( "to" AdditiveExpr )?
fn parse_range_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, from) = parse_additive_expr(input)?;

    let check = ws1_tag_ws1("to", input);
    if check.is_ok() {
        let input = check?.0;

        let (input, till) = parse_additive_expr(input)?;

        found_expr(input, Box::new(Range { from, till }))
    } else {
        Ok((input, from))
    }
}

// [88]    	AdditiveExpr 	   ::=    	MultiplicativeExpr ( ("+" | "-") MultiplicativeExpr )*
fn parse_additive_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, operand) = parse_multiplicative_expr(input)?;

    let mut left = operand;

    let mut current_input = input;
    loop {
        let check = alt((
            preceded(ws, tag("+")),
            preceded(ws1, tag("-")),
            // lookahead for '-' and digits after without whitespace before it
            terminated(tag("-"), peek(digit1))
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

            left = Box::new(Binary { left, operator, right })
        } else {
            break
        }
    }
    Ok((current_input, left))
}

// [89]    	MultiplicativeExpr 	   ::=    	UnionExpr ( ("*" | "div" | "idiv" | "mod") UnionExpr )*
fn parse_multiplicative_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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

            left = Box::new(Binary { left, operator, right });
        } else {
            break
        }
    }
    Ok((current_input, left))
}

// [90]    	UnionExpr 	   ::=    	IntersectExceptExpr ( ("union" | "|") IntersectExceptExpr )*
fn parse_union_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, expr) = parse_intersect_except_expr(input)?;

    let mut exprs = vec![];
    exprs.push(expr);

    let mut current_input = input;
    loop {
        let check = delimited(ws1, alt((tag("union"), tag("|"))), ws1)(current_input);
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
        found_expr(current_input, Box::new(Union { exprs }))
    }
}

// [91]    	IntersectExceptExpr 	   ::=    	InstanceofExpr ( ("intersect" | "except") InstanceofExpr )*
fn parse_intersect_except_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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

            left = Box::new(IntersectExcept { left, is_intersect, right })
        }
    }
    Ok((current_input, left))
}

// [92]    	InstanceofExpr 	   ::=    	TreatExpr ( "instance" "of" SequenceType )?
fn parse_instanceof_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, expr) = parse_treat_expr(input)?;

    let check = ws1_tag_ws1("instance", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("of", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Box::new(InstanceOf { expr, st } ))
    }
}

// [93]    	TreatExpr 	   ::=    	CastableExpr ( "treat" "as" SequenceType )?
fn parse_treat_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, expr) = parse_castable_expr(input)?;

    let check = ws1_tag_ws1("treat", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_sequence_type(input)?;

        found_expr(input, Box::new(Treat { expr, st } ))
    }
}

// [94]    	CastableExpr 	   ::=    	CastExpr ( "castable" "as" SingleType )?
fn parse_castable_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, expr) = parse_cast_expr(input)?;

    let check = ws1_tag_ws1("castable", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Box::new(Castable { expr, st } ))
    }
}

// [95]    	CastExpr 	   ::=    	ArrowExpr ( "cast" "as" SingleType )?
fn parse_cast_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, expr) = parse_arrow_expr(input)?;

    let check = ws1_tag_ws1("cast", input);
    if check.is_err() {
        Ok((input, expr))
    } else {
        let (input, _) = check?;
        let (input, _) = tag_ws1("as", input)?;

        let (input, st) = parse_single_type(input)?;

        found_expr(input, Box::new(Cast { expr, st } ))
    }
}

// [96]    	ArrowExpr 	   ::=    	UnaryExpr ( "=>" ArrowFunctionSpecifier ArgumentList )*
// [127]    	ArrowFunctionSpecifier 	   ::=    	EQName | VarRef | ParenthesizedExpr
fn parse_arrow_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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

// [97]    	UnaryExpr 	   ::=    	("-" | "+")* ValueExpr
// [98]    	ValueExpr 	   ::=    	ValidateExpr | ExtensionExpr | SimpleMapExpr
fn parse_unary_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {

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

    let (input, expr) = alt((
        parse_validate_expr, parse_extension_expr, parse_simple_map_expr
    ))(current_input)?;
    if let Some(sign_is_positive) = is_positive {
        found_expr(input, Box::new(Unary { expr, sign_is_positive }))
    } else {
        Ok((input, expr))
    }
}

// [102]    	ValidateExpr 	   ::=    	"validate" (ValidationMode | ("type" TypeName))? "{" Expr "}"
// [103]    	ValidationMode 	   ::=    	"lax" | "strict"
fn parse_validate_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        preceded(
            tuple((ws, tag("validate"), ws1)),
            tuple((
                opt(
                    alt((
                        map(alt((tag("lax"), tag("strict"))), |name| ValidationMode::from(name)),
                        map(preceded(tag("type"), parse_type_name), |type_name| ValidationMode::Type(type_name))
                    ))
                ),
                delimited(
                tuple((ws, tag("{"))),
                parse_expr,
                tuple((ws, tag("}")))
                )
            ))
        ),
        |(mode, expr)| ValidateExpr::boxed(mode, expr)
    )(input)
}

// [104]    	ExtensionExpr 	   ::=    	Pragma+ "{" Expr? "}"
fn parse_extension_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        tuple((
            many1(parse_pragma),
            opt(delimited(
                tuple((ws, tag("{"))),
                parse_expr,
                tuple((ws, tag("}")))
            ))
        )),
        |(pragmas, expr)| ExtensionExpr::boxed(pragmas, expr)
    )(input)
}

// ws: explicit
// [105]    	Pragma 	   ::=    	"(#" S? EQName (S PragmaContents)? "#)"
// [106]    	PragmaContents 	   ::=    	(Char* - (Char* '#)' Char*))
fn parse_pragma(input: &str) -> IResult<&str, Pragma, CustomError<&str>> {
    map(
        delimited(
            tuple((tag("(#"), ws)),
            tuple((parse_eqname, opt(preceded(ws1, is_not("#"))))),
            tag("#)")
        ),
        |(name, content)| {
            if let Some(content) = content {
                Pragma { name, content: Some(content.to_string()) }
            } else {
                Pragma { name, content: None }
            }
        }
    )(input)
}

// [107]    	SimpleMapExpr 	   ::=    	PathExpr ("!" PathExpr)*
fn parse_simple_map_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, mut exprs) = separated_list1(
        tuple((ws, tag("!"), ws)),
        parse_path_expr
    )(input)?;

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((input, expr))
    } else {
        Ok((input, Box::new(SimpleMap { exprs })))
    }
}

// [108]    	PathExpr 	   ::=    	("/" RelativePathExpr?) | ("//" RelativePathExpr) | RelativePathExpr
fn parse_path_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let check = alt((tag("//"), tag("/")))(input);
    if check.is_ok() {
        let (input, steps) = check?;
        match parse_relative_path_expr(input) {
            Ok((input, expr)) => {
                let initial_node_sequence = match steps {
                    "/" => INS::Root,
                    "//" => INS::RootDescendantOrSelf,
                    _ => panic!("internal error")
                };
                return found_expr(input, Box::new(InitialPath { initial_node_sequence, expr }));
            }
            Err(nom::Err::Failure(code)) => return Err(nom::Err::Failure(code)),
            _ => {
                if steps == "/" {
                    return found_expr(input, Box::new(Root {} ))
                } else {
                    todo!("error?")
                }
            }
        }
    }

    parse_relative_path_expr(input)
}

// [109]    	RelativePathExpr 	   ::=    	StepExpr (("/" | "//") StepExpr)*
fn parse_relative_path_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let mut exprs = vec![];

    let (input, expr) = parse_step_expr(input)?;
    let mut current_input = input;

    exprs.push(expr);

    loop {
        let check = preceded(ws, alt((tag("//"), tag("/")) ))(current_input);
        if check.is_ok() {
            let (input, steps) = check?;
            current_input = input;

            let initial_node_sequence = match steps {
                "/" => None,
                "//" => Some(INS::DescendantOrSelf),
                _ => panic!("internal error")
            };

            let (input, expr) = parse_step_expr(current_input)?;
            current_input = input;

            exprs.push(Box::new(Path { initial_node_sequence, expr }))
        } else {
            break
        }
    }

    if exprs.len() == 1 {
        let expr = exprs.remove(0);
        Ok((current_input, expr))
    } else {
        found_expr(current_input, Steps::new(exprs))
    }
}

// [110]    	StepExpr 	   ::=    	PostfixExpr | AxisStep
fn parse_step_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    alt((parse_postfix_expr, parse_axis_step))(input)
}

// [111]    	AxisStep 	   ::=    	(ReverseStep | ForwardStep) PredicateList
fn parse_axis_step(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    map(
        tuple((
                  alt((parse_reverse_step, parse_forward_step)),
                  parse_predicate_list
              )),
        |(step, predicates)| AxisStep::boxed(step, predicates)
    )(input)
}

// [112]    	ForwardStep 	   ::=    	(ForwardAxis NodeTest) | AbbrevForwardStep
fn parse_forward_step(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let check = parse_forward_axis(input);
    if check.is_ok() {
        let (input, axis) = check?;
        let (input, test) = parse_node_test(input)?;

        found_expr(input, Box::new(ForwardStep { axis, test } ))
    } else {
        parse_abbrev_forward_step(input)
    }
}

// [113]    	ForwardAxis 	   ::=    	("child" "::")
// | ("descendant" "::")
// | ("attribute" "::")
// | ("self" "::")
// | ("descendant-or-self" "::")
// | ("following-sibling" "::")
// | ("following" "::")
fn parse_forward_axis(input: &str) -> IResult<&str, Axis, CustomError<&str>> {
    map(
        preceded(
            ws,
            terminated(
                alt((
                    tag("self"),
                    tag("attribute"),
                    tag("child"),
                    tag("descendant-or-self"),
                    tag("descendant"),
                    tag("following-sibling"),
                    tag("following"),
                )),
                tag("::")
            )
        ),
        |axis| {
            match axis {
                "self" => Axis::ForwardSelf,
                "attribute" => Axis::ForwardAttribute,
                "child" => Axis::ForwardChild,
                "descendant-or-self" => Axis::ForwardDescendantOrSelf,
                "descendant" => Axis::ForwardDescendant,
                "following-sibling" => Axis::ForwardFollowingSibling,
                "following" => Axis::ForwardFollowing,
                _ => panic!("internal error")
            }
        }
    )(input)
}

// [114]    	AbbrevForwardStep 	   ::=    	"@"? NodeTest
fn parse_abbrev_forward_step(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let check = ws_tag("@", input);
    let (input, axis) = if check.is_ok() {
        let (input, _) = check?;
        (input, Axis::ForwardAttribute)
    } else {
        (input, Axis::ForwardChild)
    };

    let (input, test) = parse_node_test(input)?;

    found_expr(input, Box::new(ForwardStep { axis, test } ))
}

// [115]    	ReverseStep 	   ::=    	(ReverseAxis NodeTest) | AbbrevReverseStep
fn parse_reverse_step(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let check = parse_reverse_axis(input);
    if check.is_ok() {
        let (input, axis) = check?;
        let (input, test) = parse_node_test(input)?;

        found_expr(input, Box::new(ForwardStep { axis, test } ))
    } else {
        parse_abbrev_reverse_step(input)
    }
}

// [116]    	ReverseAxis 	   ::=    	("parent" "::")
// | ("ancestor" "::")
// | ("preceding-sibling" "::")
// | ("preceding" "::")
// | ("ancestor-or-self" "::")
fn parse_reverse_axis(input: &str) -> IResult<&str, Axis, CustomError<&str>> {
    map(
        preceded(
            ws,
            terminated(
                alt((
                    tag("parent"),
                    tag("ancestor"),
                    tag("preceding-sibling"),
                    tag("preceding"),
                    tag("ancestor-or-self"),
                )),
                tag("::")
            )
        ),
        |axis| {
            match axis {
                "parent" => Axis::ReverseParent,
                "ancestor" => Axis::ReverseAncestor,
                "preceding-sibling" => Axis::ReversePrecedingSibling,
                "preceding" => Axis::ReversePreceding,
                "ancestor-or-self" => Axis::ReverseAncestorOrSelf,
                _ => panic!("internal error")
            }
        }
    )(input)
}

// [117]    	AbbrevReverseStep 	   ::=    	".."
fn parse_abbrev_reverse_step(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("..")(input)?;

    Ok((input, NodeParent::boxed()))
}

// [118]    	NodeTest 	   ::=    	KindTest | NameTest
// TODO: parse_one_of!(parse_node_test, parse_kind_test, parse_name_test);
fn parse_node_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    alt((parse_kind_test, parse_name_test))(input)
}

// [119]    	NameTest 	   ::=    	EQName | Wildcard
// [120]    	Wildcard 	   ::=    	"*"
// | (NCName ":*")
// | ("*:" NCName)
// TODO: | (BracedURILiteral "*") 	/* ws: explicit */
fn parse_name_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    let (input, qname) = alt((
        map(
            tuple((parse_braced_uri_literal, tag("*"))),
            |(uri, _)| QName { prefix: None, url: Some(uri.to_string()), local_part: "*".to_string() }
        ),
        map(
            tuple((tag("*:"), parse_ncname)),
            |(_, name)| QName { prefix: Some("*".to_string()), url: None, local_part: name.to_string() }
        ),
        map(
            tag("*"),|_| QName::wildcard()
        ),
        map(
            tuple((parse_ncname, tag(":*"))),
            |(prefix, _)| QName { prefix: Some(prefix.to_string()), url: None, local_part: "*".to_string() }
        ),
        parse_eqname
    ))(input)?;

    // let check = parse_eqname(input);
    // let (input, qname) = if check.is_ok() {
    //     let (input, name) = check?;
    //     (input, name)
    // } else {
    //     let check = tag("*:")(input);
    //     if check.is_ok() {
    //         let (input, _) = check?;
    //         let (input, name) = parse_ncname(input)?;
    //         (input, QName::new("*".to_string(), name))
    //     } else {
    //         let check = tag("*")(input);
    //         if check.is_ok() {
    //             let (input, _) = check?;
    //             (input, QName::new("*".to_string(), "*".to_string()))
    //         } else {
    //             let (input, prefix) = parse_ncname(input)?;
    //             let (input, _) = tag(":*")(input)?;
    //
    //             (input, QName::new(prefix, "*".to_string()))
    //         }
    //     }
    // };

    Ok((input, NameTest::boxed(qname)))
}

// [121]    	PostfixExpr 	   ::=    	PrimaryExpr (Predicate | ArgumentList | TODO: Lookup)*
fn parse_postfix_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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
            let check = parse_argument_list(current_input);
            if check.is_ok() {
                let (input, arguments) = check?;
                current_input = input;

                suffix.push(PrimaryExprSuffix { predicate: None, argument_list: Some(arguments), lookup: None })
            } else {
                let check = parse_lookup(current_input);
                if check.is_ok() {
                    let (input, lookup) = check?;
                    current_input = input;

                    suffix.push(lookup)
                } else {
                    break;
                }
            }
        }
    }

    if suffix.len() == 0 {
        Ok((current_input, primary))
    } else {
        found_expr(
            current_input,
            Box::new(Postfix { primary, suffix })
        )
    }
}

// [122]    	ArgumentList 	   ::=    	"(" (Argument ("," Argument)*)? ")"
fn parse_argument_list(input: &str) -> IResult<&str, Vec<Box<dyn Expression>>, CustomError<&str>> {
    let (input, _) = ws_tag("(", input)?;

    let (input, arguments) = parse_arguments(input)?;

    let (input, _) = ws_tag(")", input)?;

    Ok((input, arguments))
}

// [123]    	PredicateList 	   ::=    	Predicate*
fn parse_predicate_list(input: &str) -> IResult<&str, Vec<PrimaryExprSuffix>, CustomError<&str>> {
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

    Ok((current_input, predicates))
}

// [124]    	Predicate 	   ::=    	"[" Expr "]"
fn parse_predicate(input: &str) -> IResult<&str, PrimaryExprSuffix, CustomError<&str>> {
    let input = ws_tag("[", input)?.0;

    let (input, expr) = parse_expr_single(input)?;
//    let (input, expr) = parse_expr(input)?;

    let input = ws_tag("]", input)?.0;

    Ok((input, PrimaryExprSuffix { predicate: Some(expr), argument_list: None, lookup: None }))
}

// [125]    	Lookup 	   ::=    	"?" KeySpecifier
// [126]    	KeySpecifier 	   ::=    	NCName | IntegerLiteral | ParenthesizedExpr | "*"
fn parse_lookup(input: &str) -> IResult<&str, PrimaryExprSuffix, CustomError<&str>> {
    let input = ws_tag("?", input)?.0;

    let (input, lookup) = alt((
        parse_ncname_expr,
        parse_integer_literal,
        parse_parenthesized_expr
    ))(input)?;

    Ok((input, PrimaryExprSuffix { predicate: None, argument_list: None, lookup: Some(lookup) }))
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
parse_one_of!(parse_primary_expr,
    parse_literal,
    parse_var_ref,
    parse_parenthesized_expr,
    parse_context_item_expr,
    parse_function_call,
    parse_node_constructor,
    parse_function_item_expr,
    parse_map_constructor,
    parse_array_constructor,
);

// [131]    	VarRef 	   ::=    	"$" VarName
fn parse_var_ref(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_var_name(input)?;

    found_expr(input, Box::new(VarRef { name }))
}

// [132]    	VarName 	   ::=    	EQName
fn parse_var_name(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    parse_eqname(input)
}

// [133]    	ParenthesizedExpr 	   ::=    	"(" Expr? ")"
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {

    let (input, _) = ws_tag("(", input)?;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        let (input, result) = check?;
        (input, Sequence::new(result))
    } else {
        (input, Sequence::empty())
    };

    let (input, _) = ws_tag(")", input)?;

    found_expr(input, expr)
}

// [134]    	ContextItemExpr 	   ::=    	"."
fn parse_context_item_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tuple((ws, tag(".")))(input)?;

    //workaround: lookahead for ".." case
    let check = is_a::<&str, &str, Error<&str>>(".")(input);
    if check.is_ok() {
        // TODO: is it correct?
        Err(nom::Err::Error(nom::error::ParseError::from_char(input, ' ')))
    } else {
        found_expr(input, Box::new(ContextItem {}))
    }
}

// [137]    	FunctionCall 	   ::=    	EQName ArgumentList
fn parse_function_call(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, function) = parse_function_name(input)?;
    let (input, arguments) = parse_argument_list(input)?;

    //workaround: lookahead for inline function
    let check = ws_tag("{", input);
    if check.is_ok() {
        // TODO: is it correct?
        Err(nom::Err::Error(nom::error::ParseError::from_char(input, ' ')))
    } else {
        found_expr(input, Box::new(Call { function, arguments }))
    }
}

fn parse_function_name(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    let (input, name) = parse_eqname(input)?;

    // Reserved Function Names
    if name.prefix.is_none() {
        match name.local_part.as_str() {
            "array" |
            "attribute" |
            "comment" |
            "document-node" |
            "element" |
            "empty-sequence" |
            "function" |
            "if" |
            "item" |
            "map" |
            "namespace-node" |
            "node" |
            "processing-instruction" |
            "schema-attribute" |
            "schema-element" |
            "switch" |
            "text" |
            "typeswitch" => {
                return Err(nom::Err::Error(CustomError::XPST0003));
            }
            _ => {}
        }
    }
    Ok((input, name))
}

// [138]    	Argument 	   ::=    	ExprSingle TODO: | ArgumentPlaceholder
// TODO: (Argument ("," Argument)*)?
fn parse_arguments(input: &str) -> IResult<&str, Vec<Box<dyn Expression>>, CustomError<&str>> {
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
                    return Ok((current_input, arguments));
                }
                current_input = tmp?.0;

                let (input, argument) = parse_expr_single(current_input)?;
                current_input = input;

                arguments.push(argument);
            }
        },
        Err(nom::Err::Failure(code)) => Err(nom::Err::Failure(code)),
        _ => {
            Ok((current_input, arguments))
        }
    }
}

// [167]    	FunctionItemExpr 	   ::=    	NamedFunctionRef | InlineFunctionExpr
parse_one_of!(parse_function_item_expr,
    parse_named_function_ref,
    parse_inline_function_expr,
);

// [168]    	NamedFunctionRef 	   ::=    	EQName "#" IntegerLiteral
fn parse_named_function_ref(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let (input, name) = parse_function_name(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, arity) = parse_integer_literal(input)?;

    found_expr(input, Box::new(NamedFunctionRef { name, arity }))
}

// [169]    	InlineFunctionExpr 	   ::=    	Annotation* "function" "(" ParamList? ")" ("as" SequenceType)? FunctionBody
// [35]    	FunctionBody 	   ::=    	EnclosedExpr
fn parse_inline_function_expr(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {

    // TODO: Annotation*

    let (input, _) = ws_tag("function", input)?;
    let (input, _) = ws_tag("(", input)?;

    let (input, arguments) = match parse_param_list(input) {
        Ok((input, params)) => (input, params),
        Err(nom::Err::Failure(code)) => return Err(nom::Err::Failure(code)),
        Err(_) => (input, vec![])
    };

    let (input, _) = ws_tag(")", input)?;

    let (input, st) = opt(
        preceded(
            tuple((ws1, tag("as"), ws1)),
            parse_sequence_type
        )
    )(input)?;

    let (input, body) = parse_enclosed_expr(input)?;

    found_expr(input, Box::new(Function { arguments, st, body }))
}

// [170]    	MapConstructor 	   ::=    	"map" "{" (MapConstructorEntry ("," MapConstructorEntry)*)? "}"
fn parse_map_constructor(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let input = ws_tag("map", input)?.0;

    let input = ws_tag("{", input)?.0;

    let mut entries = vec![];

    let mut current_input = input;
    loop {
        let result = parse_map_constructor_entry(current_input);
        if result.is_err() {

            current_input = tag("}")(input)?.0;

            return found_expr(current_input, Box::new(Map { entries }));
        }
        let (input, entry) = result?;

        entries.push(entry);

        let input = ws(input)?.0;

        let tmp = tag(",")(input);
        if tmp.is_err() {

            current_input = tag("}")(input)?.0;

            return found_expr(current_input, Box::new(Map { entries }));
        }
        current_input = tmp?.0;
    }
}

// [171]    	MapConstructorEntry 	   ::=    	MapKeyExpr ":" MapValueExpr
// [172]    	MapKeyExpr 	   ::=    	ExprSingle
// [173]    	MapValueExpr 	   ::=    	ExprSingle
fn parse_map_constructor_entry(input: &str) -> IResult<&str, MapEntry, CustomError<&str>> {
    let (input, key) = parse_expr_single(input)?;

    let input = ws_tag(":", input)?.0;

    let (input, value) = parse_expr_single(input)?;

    Ok((input, MapEntry { key, value }))
}

// [174]    	ArrayConstructor 	   ::=    	SquareArrayConstructor | CurlyArrayConstructor
parse_one_of!(parse_array_constructor,
    parse_square_array_constructor, parse_curly_array_constructor,
);

// [175]    	SquareArrayConstructor 	   ::=    	"[" (ExprSingle ("," ExprSingle)*)? "]"
fn parse_square_array_constructor(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
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

    found_expr(input, Box::new(SquareArrayConstructor { items: exprs }))
}

// [176]    	CurlyArrayConstructor 	   ::=    	"array" EnclosedExpr
fn parse_curly_array_constructor(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = ws_tag("array", input)?;

    let (input, expr) = parse_enclosed_expr(input)?;

    found_expr(input, Box::new(CurlyArrayConstructor { expr }))
}

// [182]    	SingleType 	   ::=    	SimpleTypeName "?"?
// [205]    	SimpleTypeName 	   ::=    	TypeName
// [206]    	TypeName 	   ::=    	EQName
fn parse_single_type(input: &str) -> IResult<&str, SequenceType, CustomError<&str>> {
    let (input, name) = parse_eqname(input)?;

    let check = tag("?")(input);
    let (input, occurrence_indicator) = if check.is_ok() {
        let (input, _) = check?;

        (input, OccurrenceIndicator::ZeroOrOne)
    } else {
        (input, OccurrenceIndicator::ExactlyOne)
    };

    Ok((input, SequenceType { item_type: ItemType::AtomicOrUnionType(name), occurrence_indicator }))
}

// [183]    	TypeDeclaration 	   ::=    	"as" SequenceType
fn parse_type_declaration(input: &str) -> IResult<&str, SequenceType, CustomError<&str>> {
    let (input, _) = ws_tag("as", input)?;

    parse_sequence_type(input)
}

// [184]    	SequenceType 	   ::=    	("empty-sequence" "(" ")")
// | (ItemType OccurrenceIndicator?)
// [185]    	OccurrenceIndicator 	   ::=    	"?" | "*" | "+"
fn parse_sequence_type(input: &str) -> IResult<&str, SequenceType, CustomError<&str>> {
    let (input, _) = ws(input)?;
    let check = tag("empty-sequence")(input);
    if check.is_ok() {
        let input = check?.0;

        let input = ws_tag("(", input)?.0;
        let input = ws_tag(")", input)?.0;

        // TODO this it workaround, code it better
        Ok((input, SequenceType { item_type: ItemType::SequenceEmpty, occurrence_indicator: OccurrenceIndicator::ZeroOrMore }))
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

        Ok((input, SequenceType { item_type, occurrence_indicator }))
    }
}

// [186]    	ItemType 	   ::=    	KindTest | ("item" "(" ")") | TODO FunctionTest | MapTest | ArrayTest | AtomicOrUnionType | ParenthesizedItemType
fn parse_item_type(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    let check = parse_kind_test(input);
    if check.is_ok() {
        let (input, test) = check?;
        Ok((input, ItemType::Node(test)))
    } else {
        alt((parse_item, parse_array_test, parse_function_test, parse_atomic_or_union_type))(input)
    }
}

fn parse_item(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    map(
        tuple((ws, tag("item"), ws, tag("("), ws, tag(")"))),
        |_| ItemType::Item
    )(input)
}

// [187]    	AtomicOrUnionType 	   ::=    	EQName
fn parse_atomic_or_union_type(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    let (input, name) = parse_eqname(input)?;

    Ok((input, ItemType::AtomicOrUnionType(name)))
}

// [188]    	KindTest 	   ::=    	DocumentTest
// | ElementTest
// | AttributeTest
// | SchemaElementTest
// | SchemaAttributeTest
// | PITest
// | CommentTest
// | TextTest
// | NamespaceNodeTest
// | AnyKindTest
fn parse_kind_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    alt((
        parse_document_test,
        parse_element_test,
        parse_attribute_test,
        parse_schema_element_test,
        parse_schema_attribute_test,
        parse_pi_test,
        parse_comment_test,
        parse_text_test,
        parse_namespace_node_test,
        parse_any_kind_test,
    ))(input)
}

// [189]    	AnyKindTest 	   ::=    	"node" "(" ")"
fn parse_any_kind_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        tuple((ws, tag("node"), ws, tag("("), ws, tag(")"))),
        |_| AnyKindTest::boxed()
    )(input)
}

// [190]    	DocumentTest 	   ::=    	"document-node" "(" (ElementTest | SchemaElementTest)? ")"
fn parse_document_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("document-node"), ws, tag("("), ws)),
            opt(alt((parse_element_test, parse_schema_element_test))),
            tuple((ws, tag(")")))
        ),
        |test| DocumentTest::boxed(test)
    )(input)
}

// [191]    	TextTest 	   ::=    	"text" "(" ")"
fn parse_text_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        tuple((ws, tag("text"), ws, tag("("), ws, tag(")"))),
        |_| TextTest::boxed()
    )(input)
}

// [192]    	CommentTest 	   ::=    	"comment" "(" ")"
fn parse_comment_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        tuple((ws, tag("comment"), ws, tag("("), ws, tag(")"))),
        |_| CommentTest::boxed()
    )(input)
}

// [193]    	NamespaceNodeTest 	   ::=    	"namespace-node" "(" ")"
fn parse_namespace_node_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        tuple((ws, tag("namespace-node"), ws, tag("("), ws, tag(")"))),
        |_| NamespaceNodeTest::boxed()
    )(input)
}

// [194]    	PITest 	   ::=    	"processing-instruction" "(" (NCName | StringLiteral)? ")"
fn parse_pi_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("processing-instruction"), ws, tag("("), ws)),
            opt(alt((parse_ncname_expr, parse_string_literal))),
            tuple((ws, tag(")")))
        ),
        |name| PITest::boxed(name)
    )(input)
}

// [199]    	ElementTest 	   ::=    	"element" "(" (ElementNameOrWildcard ("," TypeName "?"?)?)? ")"
fn parse_element_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("element"), ws, tag("("), ws)),
            opt(tuple((
                parse_element_name_or_wildcard,
                opt(preceded(tuple((ws, tag(","), ws)), parse_type_name))
            ))),
            tuple((ws, tag(")")))
        ),
        |param| {
            match param {
                Some((name, type_annotation)) => {
                    ElementTest::boxed(Some(name), type_annotation)
                },
                None => ElementTest::boxed(None, None)
            }
        }
    )(input)
}

// [201]    	SchemaElementTest 	   ::=    	"schema-element" "(" ElementDeclaration ")"
// [202]    	ElementDeclaration 	   ::=    	ElementName
fn parse_schema_element_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("schema-element"), ws, tag("("), ws)),
            parse_eqname,
            tuple((ws, tag(")")))
        ),
        |name| SchemaElementTest::boxed(name)
    )(input)
}

// [195]    	AttributeTest 	   ::=    	"attribute" "(" (AttribNameOrWildcard ("," TypeName)?)? ")"
fn parse_attribute_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("attribute"), ws, tag("("), ws)),
            opt(tuple((
                parse_attrib_name_or_wildcard,
                opt(preceded(tuple((ws, tag(","), ws)), parse_type_name))
            ))),
            tuple((ws, tag(")")))
        ),
        |param| {
            match param {
                Some((name, type_annotation)) => {
                    AttributeTest::boxed(Some(name), type_annotation)
                },
                None => AttributeTest::boxed(None, None)
            }
        }
    )(input)
}

// [197]    	SchemaAttributeTest 	   ::=    	"schema-attribute" "(" AttributeDeclaration ")"
// [198]    	AttributeDeclaration 	   ::=    	AttributeName
// [203]    	AttributeName 	   ::=    	EQName
fn parse_schema_attribute_test(input: &str) -> IResult<&str, Box<dyn NodeTest>, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("schema-attribute"), ws, tag("("), ws)),
            parse_eqname,
            tuple((ws, tag(")")))
        ),
        |name| SchemaAttributeTest::boxed(name)
    )(input)
}

// [200]    	ElementNameOrWildcard 	   ::=    	ElementName | "*"
// [204]    	ElementName 	   ::=    	EQName
fn parse_element_name_or_wildcard(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    parse_eqname_or_wildcard(input)
}

// [196]    	AttribNameOrWildcard 	   ::=    	AttributeName | "*"
// [203]    	AttributeName 	   ::=    	EQName
fn parse_attrib_name_or_wildcard(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    parse_eqname_or_wildcard(input)
}

fn parse_eqname_or_wildcard(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    let check = parse_eqname(input);
    if check.is_ok() {
        check
    } else {
        let (input, _) = tag("*")(input)?;
        Ok((input, QName::wildcard()))
    }
}

// [206]    	TypeName 	   ::=    	EQName
fn parse_type_name(input: &str) -> IResult<&str, QName, CustomError<&str>> {
    parse_eqname(input)
}

// [207]    	FunctionTest 	   ::=    	Annotation* (AnyFunctionTest | TypedFunctionTest)
// [208]    	AnyFunctionTest 	   ::=    	"function" "(" "*" ")"
// [209]    	TypedFunctionTest 	   ::=    	"function" "(" (SequenceType ("," SequenceType)*)? ")" "as" SequenceType
fn parse_function_test(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    map(
        tuple((
            delimited(
                tuple((ws, tag("function"), ws, tag("("))),
                alt((
                    map(tag("*"), |_| None),
                    opt(
                        map(
                            tuple((
                                parse_sequence_type,
                                many0(preceded(tag(","), parse_sequence_type))
                            )),
                            |(fst, rest)| {
                                let mut args = Vec::with_capacity(1 + rest.len());
                                args.push(fst);
                                for arg in rest {
                                    args.push(arg);
                                }
                                args
                            }
                        )
                    ),
                )),
                tuple((ws, tag(")")))
            ),
            opt(
                preceded(
                    tuple((ws, tag("as"))),
                    map(parse_sequence_type, |st| Box::new(st))
                )
            )
        )),
        |(args, st)| ItemType::Function { args, st }
    )(input)
}


// [213]    	ArrayTest 	   ::=    	AnyArrayTest | TypedArrayTest
// [214]    	AnyArrayTest 	   ::=    	"array" "(" "*" ")"
// [215]    	TypedArrayTest 	   ::=    	"array" "(" SequenceType ")"
fn parse_array_test(input: &str) -> IResult<&str, ItemType, CustomError<&str>> {
    map(
        delimited(
            tuple((ws, tag("array"), ws, tag("("))),
            alt((
                map(tag("*"), |_| None),
                map(parse_sequence_type, |st| Some(Box::new(st)))
            )),
            tuple((ws, tag(")")))
        ),
        |st| ItemType::Array(st)
    )(input)
}

// [217]    	URILiteral 	   ::=    	StringLiteral
fn parse_uri_literal(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    parse_string_literal(input)
}

fn parse_uri_literal_as_string(input: &str) -> IResult<&str, String, CustomError<&str>> {
    parse_string_literal_as_string(input)
}

// [226]    	EscapeQuot 	   ::=    	'""'
fn parse_escape_quot(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("\"\"")(input)?;

    found_expr(input, Box::new(EscapeQuot{}))
}

// [227]    	EscapeApos 	   ::=    	"''"
fn parse_escape_apos(input: &str) -> IResult<&str, Box<dyn Expression>, CustomError<&str>> {
    let (input, _) = tag("''")(input)?;

    found_expr(input, Box::new(EscapeApos{}))
}