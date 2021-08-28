use std::boxed::Box;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until, is_not},
    character::complete::{multispace0, multispace1}
};
use nom::character::complete::one_of;

mod macros;
use crate::parse_sequence;
use crate::parse_one_of;

use crate::namespaces::*;
use nom::error::ParseError;
use std::fmt::Error;
use crate::value::QName;
use crate::fns::Param;

const DEBUG: bool = false;

fn found_statements(input: &str, program: Vec<Statement>) -> IResult<&str, Vec<Statement>> {
    Ok((input, program))
}

fn found_statement(input: &str, statement: Statement) -> IResult<&str, Statement> {
    Ok((input, statement))
}

fn found_exprs(input: &str, exprs: Vec<Expr>) -> IResult<&str, Vec<Expr>> {
    Ok((input, exprs))
}

fn found_expr(input: &str, expr: Expr) -> IResult<&str, Expr> {
    if DEBUG {
        println!("\nfound: {:?}\ninput: {:?}", expr, input);
    }
    Ok((input, expr))
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression (Expr),
    Let { name: String, value: Expr},
    Return { value: Expr },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Ident(String),

    Boolean(bool),
    Integer(i128),
    String(String),

    Item,

    ContextItem,

    Sequence(Vec<Statement>),
    SequenceEmpty(),
    Range { from: Box<Expr>, till: Box<Expr> },
    Predicate(Box<Expr>),
//    Predicates(Vec<Statement>), // TODO: can it be covered by Sequence(Predicate)?

    Postfix { primary: Box<Expr>, suffix: Vec<Expr> },

    Node { name: QName, attributes: Vec<Expr>, children: Vec<Expr> },
    Attribute { name: QName, value: Box<Expr> },
    NodeText(String),
    NodeComment(String),
    NodePI { target: QName, content: String },

    Map { entries: Vec<Expr> }, // Expr because can't use MapEntry here
    MapEntry { key: Box<Expr>, value: Box<Expr> },

    SquareArrayConstructor { items: Vec<Expr> },
    CurlyArrayConstructor { exprs: Vec<Statement> },

    QName { local_part: String, url: String, prefix: String },

    Binary { left: Box<Expr>, operator: Operator, right: Box<Expr> },
    Comparison { left: Box<Expr>, operator: Operator, right: Box<Expr> },

    If { condition: Box<Expr>, consequence: Vec<Statement>, alternative: Vec<Statement> },

    ArgumentList { arguments: Vec<Expr> },
    Function { arguments: Vec<Param>, body: Vec<Statement> },
    Call { function: QName, arguments: Vec<Expr> },
    NamedFunctionRef { name: QName, arity: Box<Expr> },

    ParamList(Vec<Expr>),
    Param { name: QName, typeDeclaration: Box<Option<Expr>> },

    VarRef { name: QName },

    Or(Vec<Expr>),
    And(Vec<Expr>),
    StringConcat(Vec<Expr>),
    SimpleMap(Vec<Expr>),

    FLWOR { initialClause: Box<Expr>, returnExpr: Box<Expr> },

    LetClause { bindings: Vec<Expr> },
    LetBinding { name: QName, typeDeclaration: Box<Option<Expr>>,  value: Box<Expr>},

}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Operator {
    Unknown,

    Plus,
    Minus,
    Multiply,
    Divide,
    IDivide,

    Mod,

    Equals,
    NotEquals,
    LessThan,
    LessOrEquals,
    GreaterThan,
    GreaterOrEquals,
}

// [1]    	Module 	   ::=    	TODO: VersionDecl? (LibraryModule | MainModule)
pub fn parse(input: &str) -> IResult<&str, Vec<Statement>> {

    //TODO: [6]    	Prolog 	   ::=    	((DefaultNamespaceDecl | Setter | NamespaceDecl | Import) Separator)* ((ContextItemDecl | AnnotatedDecl | OptionDecl) Separator)*

    let (input, program) = parse_expr(input)?;

    found_statements(input, program)
}

// [33]    	ParamList 	   ::=    	Param ("," Param)*
parse_sequence!(parse_param_list, ",", parse_param, ParamList);

// [34]    	Param 	   ::=    	"$" EQName TypeDeclaration?
fn parse_param(input: &str) -> IResult<&str, Expr> {
    let (input, _) = tag("$")(input)?;
    let (input, name) = parse_eqname(input)?;
    // TODO: TypeDeclaration?

    found_expr(
        input,
        Expr::Param { name, typeDeclaration: Box::new(None)}
    )
}

// [36]    	EnclosedExpr 	   ::=    	"{" Expr? "}"
fn parse_enclosed_expr(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, _) = ws_tag("{", input)?;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        check?
    } else {
        (input, vec![])
    };

    let (input, _) = ws_tag("}", input)?;

    found_statements(input, expr)
}

// [38]    	QueryBody 	   ::=    	Expr
// [39]    	Expr 	   ::=    	ExprSingle ("," ExprSingle)*
fn parse_expr(input: &str) -> IResult<&str, Vec<Statement>> {
    let mut program = vec![];

    let mut current_input = input;
    loop {
        let (input, expr) = parse_expr_single(current_input)?;

        program.push(Statement::Expression( expr ));

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                found_statements(input, program)
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
fn parse_expr_single(input: &str) -> IResult<&str, Expr> {
    if DEBUG {
        println!("parse_expr_single: {:?}", input);
    }

    let check = parse_flwor_expr(input);
    if check.is_ok() {
        let (mut input, expr) = check?;
        return found_expr(
            input,
            expr
        )
    }

    let (input, expr) = parse_or_expr(input)?;
    found_expr(
        input,
        expr
    )
}

// [41]    	FLWORExpr 	   ::=    	InitialClause IntermediateClause* ReturnClause
// [42]    	InitialClause 	   ::=    	ForClause | LetClause | WindowClause

// [69]    	ReturnClause 	   ::=    	"return" ExprSingle
fn parse_flwor_expr(input: &str) -> IResult<&str, Expr> {
    let (input, initialClause) = parse_let_clause_expr(input)?;

    let (input, _) = ws_tag("return", input)?;

    let (input, returnExpr) = parse_expr_single(input)?;

    found_expr(
        input,
        Expr::FLWOR { initialClause: Box::new(initialClause), returnExpr: Box::new(returnExpr) }
    )
}

// [48]    	LetClause 	   ::=    	"let" LetBinding ("," LetBinding)*
fn parse_let_clause_expr(input: &str) -> IResult<&str, Expr> {

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
fn parse_let_binding_expr(input: &str) -> IResult<&str, Expr> {

    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    let check = parse_type_declaration_expr(input);
    let (input, typeDeclaration) = if check.is_ok() {
        let (input, typeDeclaration) = check?;
        (input, Some(typeDeclaration))
    } else {
        (input, None)
    };

    let (input, _) = ws_tag(":=", input)?;

    let (input, value) = parse_expr_single(input)?;

    found_expr(
        input,
        Expr::LetBinding { name, typeDeclaration: Box::new(typeDeclaration),  value: Box::new(value)}
    )
}

// [83]    	OrExpr 	   ::=    	AndExpr ( "or" AndExpr )*
parse_sequence!(parse_or_expr, "or", parse_and_expr, Or);

// [84]    	AndExpr 	   ::=    	ComparisonExpr ( "and" ComparisonExpr )*
parse_sequence!(parse_and_expr, "and", parse_comparison_expr, And);

// [85]    	ComparisonExpr 	   ::=    	StringConcatExpr ( ( TODO ValueComp
// TODO | GeneralComp
// TODO | NodeComp) StringConcatExpr )?
fn parse_comparison_expr(input: &str) -> IResult<&str, Expr> {
    parse_string_concat_expr(input)
}

// [86]    	StringConcatExpr 	   ::=    	RangeExpr ( "||" RangeExpr )*
parse_sequence!(parse_string_concat_expr, "||", parse_range_expr, StringConcat);

// [87]    	RangeExpr 	   ::=    	AdditiveExpr ( "to" AdditiveExpr )?
fn parse_range_expr(input: &str) -> IResult<&str, Expr> {
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
fn parse_binary_expr(input: &str) -> IResult<&str, Expr> {

    if DEBUG {
        println!("parse_binary_expr 1: {:?}", input);
    }

    let (input, expr) = parse_unary_expr(input)?;

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

            let (input, right) = parse_unary_expr(input)?;
            current_input = input;

            if DEBUG {
                println!("parse_binary_expr: {:?} {:?} {:?}", left, op, right);
            }

            let operator = match op {
                "=" => Operator::Equals,
                "!=" => Operator::NotEquals,
                "<" => Operator::LessThan,
                "<=" => Operator::LessOrEquals,
                ">" => Operator::GreaterThan,
                ">=" => Operator::GreaterOrEquals,

                "eq" => Operator::Equals,
                "ne" => Operator::NotEquals,
                "lt" => Operator::LessThan,
                "le" => Operator::LessOrEquals,
                "gt" => Operator::GreaterThan,
                "ge" => Operator::GreaterOrEquals,

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

// [97]    	UnaryExpr 	   ::=    	("-" | "+")* TODO: ValueExpr
// [98]    	ValueExpr 	   ::=    	TODO: ValidateExpr | TODO: ExtensionExpr | SimpleMapExpr
fn parse_unary_expr(input: &str) -> IResult<&str, Expr> {

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

    parse_simple_map_expr(current_input)
}

// [107]    	SimpleMapExpr 	   ::=    	PathExpr ("!" PathExpr)*
parse_sequence!(parse_simple_map_expr, "!", parse_path_expr, SimpleMap);

// [108]    	PathExpr 	   ::=    	TODO: ("/" RelativePathExpr?) | ("//" RelativePathExpr) | RelativePathExpr
fn parse_path_expr(input: &str) -> IResult<&str, Expr> {
    parse_relative_path_expr(input)
}

// [109]    	RelativePathExpr 	   ::=    	TODO: StepExpr (("/" | "//") StepExpr)*
fn parse_relative_path_expr(input: &str) -> IResult<&str, Expr> {
    parse_step_expr(input)
}

// [110]    	StepExpr 	   ::=    	TODO: PostfixExpr | AxisStep
fn parse_step_expr(input: &str) -> IResult<&str, Expr> {
    parse_postfix_expr(input)
}

//// [111]    	AxisStep 	   ::=    	(ReverseStep | ForwardStep) PredicateList
//// [123]    	PredicateList 	   ::=    	Predicate*
//fn parse_axis_step(input: &str) -> IResult<&str, Expr> {
//
//}

// [121]    	PostfixExpr 	   ::=    	PrimaryExpr (Predicate | TODO: ArgumentList | Lookup)*
fn parse_postfix_expr(input: &str) -> IResult<&str, Expr> {
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
fn parse_argument_list(input: &str) -> IResult<&str, Vec<Expr>> {
    let (input, _) = ws_tag("(", input)?;

    let (input, arguments) = parse_arguments(input)?;

    let (input, _) = ws_tag(")", input)?;

    found_exprs(
        input,
        arguments
    )
}

// [124]    	Predicate 	   ::=    	"[" Expr "]"
fn parse_predicate(input: &str) -> IResult<&str, Expr> {
    if DEBUG {
        println!("parse_predicate: {:?}", input);
    }

    let input = ws_tag("[", input)?.0;

    let (input, expr) = parse_expr_single(input)?;
//    let (input, expr) = parse_expr(input)?;

    let input = ws_tag("]", input)?.0;

    Ok((input, Expr::Predicate(Box::new(expr))))
}

// [128]    	PrimaryExpr 	   ::=    	Literal
//  | VarRef
//  | ParenthesizedExpr
//  | ContextItemExpr
//  | FunctionCall
//  TODO: | OrderedExpr
//  TODO: | UnorderedExpr
//  TODO: | NodeConstructor
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
    //
);

// [131]    	VarRef 	   ::=    	"$" VarName
// [132]    	VarName 	   ::=    	EQName
fn parse_var_ref(input: &str) -> IResult<&str, Expr> {
    let (input, _) = ws_tag("$", input)?;

    let (input, name) = parse_eqname(input)?;

    Ok((
        input,
        Expr::VarRef { name }
    ))
}

// [134]    	ContextItemExpr 	   ::=    	"."
fn parse_context_item_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = ws_tag(".", input)?;

    Ok((
        input,
        Expr::ContextItem
    ))
}

// [137]    	FunctionCall 	   ::=    	EQName ArgumentList
fn parse_function_call(input: &str) -> IResult<&str, Expr> {
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
fn parse_arguments(input: &str) -> IResult<&str, Vec<Expr>> {
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

// [129]    	Literal 	   ::=    	TODO: NumericLiteral | StringLiteral
fn parse_literal(input: &str) -> IResult<&str, Expr> {

    let input = ws(input)?.0;

    let result = parse_numeric_literal(input);
    if result.is_ok() {
        let (input, literal) = result?;
        return Ok((
            input,
            literal
        ))
    }

    parse_string_literal(input)
}

// [130]    	NumericLiteral 	   ::=    	IntegerLiteral TODO: | DecimalLiteral | DoubleLiteral
fn parse_numeric_literal(input: &str) -> IResult<&str, Expr> {
    let (input, number) = take_while1(is_digits)(input)?;

    Ok((
        input,
        Expr::Integer(number.parse::<i128>().unwrap())
    ))
}

// [133]    	ParenthesizedExpr 	   ::=    	"(" Expr? ")"
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr> {

    let input = ws_tag("(", input)?.0;

    let check = parse_expr(input);
    let (input, expr) = if check.is_ok() {
        let (input, result) = check?;
        (input, Expr::Sequence(result))
    } else {
        (input, Expr::SequenceEmpty())
    };

    let input = ws_tag(")", input)?.0;

    Ok((input, expr))
}

// [140]    	NodeConstructor 	   ::=    	DirectConstructor | ComputedConstructor
fn parse_node_constructor(input: &str) -> IResult<&str, Expr> {
    let result = parse_direct_constructor(input);
    if result.is_ok() {
        let (input, node) = result?;
        return Ok((
            input,
            node
        ))
    }

    parse_computed_constructor(input)
}

// TODO:
// [141]    	DirectConstructor 	   ::=    	DirElemConstructor | DirCommentConstructor | DirPIConstructor
// [142]    	DirElemConstructor 	   ::=    	"<" QName DirAttributeList ("/>" | (">" DirElemContent* "</" QName S? ">")) // ws: explicit
// [149]    	DirCommentConstructor 	   ::=    	"<!--" DirCommentContents "-->" // ws: explicit
// [150]    	DirCommentContents 	   ::=    	((Char - '-') | ('-' (Char - '-')))* // ws: explicit
// [151]    	DirPIConstructor 	   ::=    	"<?" PITarget (S DirPIContents)? "?>" // ws: explicit
// [152]    	DirPIContents 	   ::=    	(Char* - (Char* '?>' Char*)) // ws: explicit
fn parse_direct_constructor(input: &str) -> IResult<&str, Expr> {
    if DEBUG {
        println!("parse_direct_constructor {:?}", input);
    }

    let input = tag("<")(input)?.0;

    // DirCommentConstructor
    let result = tag("!--")(input);
    if result.is_ok() {
        let (input, _) = result?;
        let (input, content) = take_until("-->")(input)?;

        let input = tag("-->")(input)?.0;

        //TODO: raise error if content end by '-'

        return Ok((
            input,
            Expr::NodeComment(String::from(content))
        ))
    }

    // DirPIConstructor
    let result = tag("?")(input);
    if result.is_ok() {
        let input = result?.0;

        let (input, target) = parse_qname(input)?;

        //TODO: target must not be 'xml'

        let (input, content) = take_until("?>")(input)?;

        let input = tag("?>")(input)?.0;

        return Ok((
            input,
            Expr::NodePI { target, content: String::from(content) }
        ))
    }

    // DirElemConstructor

    // "<" QName DirAttributeList ("/>" | (">" DirElemContent* "</" QName S? ">"))

    let (input, tag_name) = parse_qname(input)?;

    let (input, attributes) = parse_attribute_list(input)?;

    let mut children = Vec::new();

    let mut current_input = input;

    let check = tag("/>")(current_input);
    if check.is_ok() {
        current_input = check?.0;

    } else {
        current_input = tag(">")(current_input)?.0;
        loop {
            if DEBUG {
                println!("parse_direct_constructor check_for_close {:?} {:?}", tag_name, current_input);
            }

            let check_for_close = tag("</")(current_input);
            if check_for_close.is_ok() {
                let (_,_) = check_for_close?;
                break;
            }

            let check = parse_dir_elem_content(current_input);
            if check.is_ok() {
                let (input, child) = check?;
                current_input = input;

                children.push(child);
            } else {
                break
            }
        }
        if DEBUG {
            println!("parse_direct_constructor close tag {:?} {:?}", tag_name, current_input);
        }

        current_input = tag("</")(current_input)?.0;

        let (input, close_tag_name) = parse_qname(current_input)?;

        current_input = ws(input)?.0;

        current_input = tag(">")(current_input)?.0;

        if tag_name != close_tag_name {
            panic!("close tag '{:?}' do not match open one '{:?}'", close_tag_name, tag_name); // TODO: better error
        }
    };

    if DEBUG {
        println!("parse_direct_constructor return {:?}", current_input);
    }

    Ok((
        current_input,
        Expr::Node { name: tag_name, attributes, children }
    ))
}

// [143]    	DirAttributeList 	   ::=    	(S (QName S? "=" S? DirAttributeValue)?)* // ws: explicit
// [144]    	DirAttributeValue 	   ::=    	('"' (EscapeQuot | QuotAttrValueContent)* '"') | ("'" (EscapeApos | AposAttrValueContent)* "'") // ws: explicit
// [145]    	QuotAttrValueContent 	   ::=    	QuotAttrContentChar | CommonContent
// [146]    	AposAttrValueContent 	   ::=    	AposAttrContentChar | CommonContent
fn parse_attribute_list(input: &str) -> IResult<&str, Vec<Expr>> {

    let mut attributes = Vec::new();

    let mut current_input = input;

    loop {
        let check = multispace1(current_input);
        if check.is_err() {
            break;
        }
        current_input = check?.0;

        if DEBUG {
            println!("parse_attribute_list ws {:?}", current_input);
        }

        let check = parse_qname(current_input);
        if check.is_ok() {
            let (input, name) = check?;
            if DEBUG {
                println!("parse_attribute_list qname {:?}", input);
            }

            let input = ws(input)?.0;
            let input = tag("=")(input)?.0;
            let input = ws(input)?.0;

            let (input, close_char) = alt((tag("\""), tag("'")))(input)?;

            if DEBUG {
                println!("parse_attribute_list {:?} {:?}", close_char, input);
            }

            let mut value = String::new();

            current_input = input;
            loop {
                let (input, content) = take_until(close_char)(current_input)?;

                if DEBUG {
                    println!("parse_attribute_list {:?}", content);
                }

                let (input, _) = tag(close_char)(input)?;
                current_input = input;

                value.push_str(content);

                let check = tag(close_char)(current_input);
                if check.is_ok() {
                    current_input = check?.0;

                    value.push_str(close_char);
                } else {
                    break;
                }
            }

            if DEBUG {
                println!("parse_attribute_list attribute {:?} = {:?}", name, value);
            }

            attributes.push(
                Expr::Attribute { name, value: Box::new(Expr::String( String::from(value) )) }
            )
        } else {
            break;
        }
    }

    if DEBUG {
        println!("parse_attribute_list return {:?}", current_input);
    }

    Ok((
        current_input,
        attributes
    ))
}

// [147]    	DirElemContent 	   ::=    	DirectConstructor TODO: | CDataSection | CommonContent | ElementContentChar
// [148]    	CommonContent 	   ::=    	PredefinedEntityRef | CharRef | "{{" | "}}" | EnclosedExpr
// [153]    	CDataSection 	   ::=    	"<![CDATA[" CDataSectionContents "]]>" // ws: explicit
// [154]    	CDataSectionContents 	   ::=    	(Char* - (Char* ']]>' Char*)) // ws: explicit
// [228]    	ElementContentChar 	   ::=    	(Char - [{}<&])
fn parse_dir_elem_content(input: &str) -> IResult<&str, Expr> {
    if DEBUG {
        println!("parse_dir_elem_content: {:?}", input);
    }

    let check = parse_direct_constructor(input);
    if check.is_ok() {
        return check
    }

//    c == '{' || c == '}' || c == '<' || c == '&'
    let (input, content) = is_not("{}<&")(input)?;

    //TODO: code others

    if DEBUG {
        println!("NodeText: {:?} {:?}", content, input);
    }

    Ok((
        input,
        Expr::NodeText(String::from(content))
    ))
}

// [225]    	PredefinedEntityRef 	   ::=    	"&" ("lt" | "gt" | "amp" | "quot" | "apos") ";"
// [66]   	CharRef	   ::=   	'&#' [0-9]+ ';' | '&#x' [0-9a-fA-F]+ ';'

// [155]    	ComputedConstructor 	   ::=    	CompDocConstructor
// | CompElemConstructor
// | CompAttrConstructor
// | CompNamespaceConstructor
// | CompTextConstructor
// | CompCommentConstructor
// | CompPIConstructor
// [156]    	CompDocConstructor 	   ::=    	"document" EnclosedExpr
// [157]    	CompElemConstructor 	   ::=    	"element" (EQName | ("{" Expr "}")) EnclosedContentExpr
// [158]    	EnclosedContentExpr 	   ::=    	EnclosedExpr
// [159]    	CompAttrConstructor 	   ::=    	"attribute" (EQName | ("{" Expr "}")) EnclosedExpr
// [160]    	CompNamespaceConstructor 	   ::=    	"namespace" (Prefix | EnclosedPrefixExpr) EnclosedURIExpr
// [161]    	Prefix 	   ::=    	NCName
// [162]    	EnclosedPrefixExpr 	   ::=    	EnclosedExpr
// [163]    	EnclosedURIExpr 	   ::=    	EnclosedExpr
// [164]    	CompTextConstructor 	   ::=    	"text" EnclosedExpr
// [165]    	CompCommentConstructor 	   ::=    	"comment" EnclosedExpr
// [166]    	CompPIConstructor 	   ::=    	"processing-instruction" (NCName | ("{" Expr "}")) EnclosedExpr
fn parse_computed_constructor(input: &str) -> IResult<&str, Expr> {
    let input = ws(input)?.0;

    let (input, name) = alt(
        ( tag("document"), tag("element"), tag("attribute"), tag("namespace"), tag("text"), tag("comment"), tag("processing-instruction")  )
    )(input)?;

    // TODO: finish it
    Ok((
        "TODO",
        Expr::String( String::from( "TODO" ))
    ))
}

// [167]    	FunctionItemExpr 	   ::=    	NamedFunctionRef | InlineFunctionExpr
parse_one_of!(parse_function_item_expr,
    parse_named_function_ref,
    parse_inline_function_expr,
);

// [168]    	NamedFunctionRef 	   ::=    	EQName "#" IntegerLiteral
fn parse_named_function_ref(input: &str) -> IResult<&str, Expr> {

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
fn parse_inline_function_expr(input: &str) -> IResult<&str, Expr> {

    // TODO: Annotation*

    let (input, _) = ws_tag("function", input)?;
    let (input, _) = ws_tag("(", input)?;

    let check = parse_param_list(input);
    let (input, arguments) = if check.is_ok() {
        let (input, expr) = check?;

        let params = match expr {
            Expr::ParamList(exprs) => {
                let mut params = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let param = match expr {
                        Expr::Param { name, typeDeclaration } => {
                            Param { name, sequenceType: None }
                        }
                        _ => panic!("expected Param but got {:?}", expr)
                    };
                    params.push(param);
                }
                params
            },
            _ => panic!("expected ParamList but got {:?}", expr)
        };

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
        Expr::Function { arguments, body }
    ))
}

// [170]    	MapConstructor 	   ::=    	"map" "{" (MapConstructorEntry ("," MapConstructorEntry)*)? "}"
fn parse_map_constructor(input: &str) -> IResult<&str, Expr> {
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
fn parse_map_constructor_entry(input: &str) -> IResult<&str, Expr> {
    let (input, key) = parse_expr_single(input)?;

    let input = ws_tag(":", input)?.0;

    let (input, value) = parse_expr_single(input)?;

    Ok((
        input,
        Expr::MapEntry { key: Box::new( key ), value: Box::new( value ) }
    ))
}

// [174]    	ArrayConstructor 	   ::=    	SquareArrayConstructor | CurlyArrayConstructor
parse_one_of!(parse_array_constructor,
    parse_square_array_constructor,
    parse_curly_array_constructor,
);

// [175]    	SquareArrayConstructor 	   ::=    	"[" (ExprSingle ("," ExprSingle)*)? "]"
fn parse_square_array_constructor(input: &str) -> IResult<&str, Expr> {
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
fn parse_curly_array_constructor(input: &str) -> IResult<&str, Expr> {
    let (input, _) = ws_tag("array", input)?;

    let (input, exprs) = parse_enclosed_expr(input)?;

    Ok((
        input,
        Expr::CurlyArrayConstructor { exprs }
    ))
}

// [183]    	TypeDeclaration 	   ::=    	"as" SequenceType
fn parse_type_declaration_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = ws_tag("as", input)?;

    parse_sequence_type_expr(input)
}

// [184]    	SequenceType 	   ::=    	("empty-sequence" "(" ")")
// | (ItemType OccurrenceIndicator?)
// TODO [185]    	OccurrenceIndicator 	   ::=    	"?" | "*" | "+"
fn parse_sequence_type_expr(input: &str) -> IResult<&str, Expr> {
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
        parse_item_type_expr(input)
    }
}

// TODO [186]    	ItemType 	   ::=    	KindTest | ("item" "(" ")") | FunctionTest | MapTest | ArrayTest | AtomicOrUnionType | ParenthesizedItemType
fn parse_item_type_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = ws_tag("item", input)?;
    let (input, _) = ws_tag("(", input)?;
    let (input, _) = ws_tag(")", input)?;

    Ok((
        input,
        Expr::Item
    ))
}

// [219]    	IntegerLiteral 	   ::=    	Digits
fn parse_integer_literal(input: &str) -> IResult<&str, Expr> {
    let (input, number) = take_while1(is_digits)(input)?;

    Ok((
        input,
        Expr::Integer(number.parse::<i128>().unwrap())
    ))
}

// [222]    	StringLiteral 	   ::=    	('"' (PredefinedEntityRef | CharRef | EscapeQuot | [^"&])* '"') | ("'" (PredefinedEntityRef | CharRef | EscapeApos | [^'&])* "'")
fn parse_string_literal(input: &str) -> IResult<&str, Expr> {
    let input = ws_tag("\"", input)?.0;

    let (input, string) = take_until("\"")(input)?;

    let input = tag("\"")(input)?.0;

    Ok((
        input,
        Expr::String( String::from(string) )
    ))
}

fn parse_name(input: &str) -> IResult<&str, String> {
    parse_ncname(input)
}

// [7]   	QName	   ::=   	PrefixedName | UnprefixedName
fn parse_qname(input: &str) -> IResult<&str, QName> {
    // use as workaround
    parse_eqname(input)
}

// [218]    	EQName 	   ::=    	QName TODO: | URIQualifiedName
fn parse_eqname(input: &str) -> IResult<&str, QName> {
    // [8]   	PrefixedName	   ::=   	Prefix ':' LocalPart
    // [9]   	UnprefixedName	   ::=   	LocalPart
    // [10]   	Prefix	   ::=   	NCName
    // [11]   	LocalPart	   ::=   	NCName

    if DEBUG {
        println!("parse_eqname: {:?}", input);
    }

    let (input, name1) = parse_ncname(input)?;

    let check = tag(":")(input);

    if check.is_ok() {
        let (input, _) = check?;

        let (input, name2) = parse_ncname(input)?;

        // TODO: resolve url from environment
        let url = if name1 == String::from(XML.prefix) {
            XML.url
        } else if name1 == String::from(SCHEMA.prefix) {
            SCHEMA.url
        } else if name1 == String::from(SCHEMA_INSTANCE.prefix) {
            SCHEMA_INSTANCE.url
        } else if name1 == String::from(XPATH_FUNCTIONS.prefix) {
            XPATH_FUNCTIONS.url
        } else if name1 == String::from(XPATH_MAP.prefix) {
            XPATH_MAP.url
        } else if name1 == String::from(XPATH_ARRAY.prefix) {
            XPATH_ARRAY.url
        } else if name1 == String::from(XPATH_MATH.prefix) {
            XPATH_MATH.url
        } else if name1 == String::from(XQUERY_LOCAL.prefix) {
            XQUERY_LOCAL.url
        } else if name1 == String::from(XQT_ERROR.prefix) {
            XQT_ERROR.url
        } else {
            ""
        };

        Ok((
            input,
            QName { local_part: name2, url: String::from(url), prefix: name1 }
        ))
    } else {
        Ok((
            input,
            QName { local_part: name1, url: String::from(""), prefix: String::from("") } // TODO: resolve namespace
        ))
    }
}

// [4]   	NCName	   ::=   	Name - (Char* ':' Char*)	/* An XML Name, minus the ":" */
fn parse_ncname(input: &str) -> IResult<&str, String> {
    let (input, name_start) = take_while_m_n(1, 1, is_name_start_char)(input)?;
    let (input, name_end) = take_while(is_name_char)(input)?;

    let mut name = String::new();
    name.push_str(name_start);
    name.push_str(name_end);

    Ok((
        input,
        name
    ))
}

//[4]   	NameStartChar	   ::=   	":" (An XML Name, minus the ":") | [A-Z] | "_" | [a-z] TODO: | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn is_name_start_char(c: char) -> bool {
   c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}

//[4a]   	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] TODO: | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
fn is_name_char(c: char) -> bool {
    is_name_start_char(c) || c == '-' || c == '.' || (c >= '0' && c <= '9')
}

//[238]    	Digits 	   ::=    	[0-9]+
fn is_digits(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn ws(input: &str) -> IResult<&str, &str> {
    multispace0(input)
}

fn ws_tag<'a>(token: &str, input: &'a str) -> IResult<&'a str, &'a str> {
    let (input, _) = multispace0(input)?;
    tag(token)(input)
}

fn expr_to_qname(expr: Expr) -> QName {
    match expr {
        Expr::QName { prefix, url, local_part } => QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", expr)
    }
}