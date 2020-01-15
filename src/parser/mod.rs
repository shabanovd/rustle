use std::boxed::Box;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until},
    character::complete::multispace0
};
use nom::character::complete::one_of;
use nom::Err::Error;

use crate::namespaces::*;

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

    Map { entries: Vec<Expr> }, // Expr because can't use MapEntry here
    MapEntry { key: Box<Statement>, value: Box<Statement> },

    QName { local_part: String, url: String, prefix: String },

    Binary { left: Box<Expr>, operator: Operator, right: Box<Expr> },
    If { condition: Box<Expr>, consequence: Vec<Statement>, alternative: Vec<Statement> },

    Function { arguments: Vec<String>, body: Vec<Statement> },
    Call { function: Box<Expr>, arguments: Vec<Statement> },
//    Call { function: Box<Expr>, arguments: Vec<Statement> },

}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    IDivide,

    Mod,

    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
}

// [1]    	Module 	   ::=    	TODO: VersionDecl? (LibraryModule | MainModule)
pub fn parse(input: &str) -> IResult<&str, Vec<Statement>> {

    //TODO: [6]    	Prolog 	   ::=    	((DefaultNamespaceDecl | Setter | NamespaceDecl | Import) Separator)* ((ContextItemDecl | AnnotatedDecl | OptionDecl) Separator)*

    let (input, program) = parse_expr(input)?;

    Ok((
        input,
        program
    ))
}

// [38]    	QueryBody 	   ::=    	Expr
// [39]    	Expr 	   ::=    	ExprSingle ("," ExprSingle)*
fn parse_expr(input: &str) -> IResult<&str, Vec<Statement>> {
    let mut program = vec![];

    let mut current_input = input;
    loop {
        let (input, expr) = parse_expr_single(current_input)?;

        program.push(expr);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                Ok((
                    input,
                    program
                ))
        }
        current_input = tmp?.0;
    }
}

// [40]    	ExprSingle 	   ::=    	TODO: FLWORExpr
//  TODO: | QuantifiedExpr
//  TODO: | SwitchExpr
//  TODO: | TypeswitchExpr
//  TODO: | IfExpr
//  TODO: | TryCatchExpr
//  TODO: | OrExpr
fn parse_expr_single(input: &str) -> IResult<&str, Statement> {
    println!("parse_expr_single: {:?}", input);
    let (input, expr) = parse_binary_expr(input)?;

    Ok((
        input,
        Statement::Expression( expr )
    ))
}

fn parse_binary_expr(input: &str) -> IResult<&str, Expr> {

    let (input, left) = parse_unary_expr(input)?;

    let input = ws(input)?.0;

    let check = alt(
        ( tag("+"), tag("-"), tag("*"), tag("div"), tag("idiv"), tag("mod")  )
    )(input);

    if check.is_ok() {
        let (input, op) = check?;

        let (input, right) = parse_unary_expr(input)?;

        let operator = match op {
            "+" => Operator::Plus,
            "-" => Operator::Minus,
            "*" => Operator::Multiply,
            "div" => Operator::Divide,
            "idiv" => Operator::IDivide,
            "mod" => Operator::Mod,
            _ => panic!("it must not happen") // TODO: raise error instead
        };

        Ok((
            input,
            Expr::Binary { left: Box::new( left ), operator, right: Box::new( right ) }
        ))

    } else {
        Ok((
            input,
            left
        ))
    }



}

// [97]    	UnaryExpr 	   ::=    	("-" | "+")* ValueExpr
// [98]    	ValueExpr 	   ::=    	ValidateExpr | ExtensionExpr | SimpleMapExpr
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

// [107]    	SimpleMapExpr 	   ::=    	PathExpr TODO: ("!" PathExpr)*
fn parse_simple_map_expr(input: &str) -> IResult<&str, Expr> {
    parse_path_expr(input)
}

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

// [121]    	PostfixExpr 	   ::=    	TODO: PrimaryExpr (Predicate | ArgumentList | Lookup)*
fn parse_postfix_expr(input: &str) -> IResult<&str, Expr> {
    parse_primary_expr(input)
}



// [128]    	PrimaryExpr 	   ::=    	Literal
//  TODO: | VarRef
//  TODO: | ParenthesizedExpr
//  TODO: | ContextItemExpr
//  | FunctionCall
//  TODO: | OrderedExpr
//  TODO: | UnorderedExpr
//  TODO: | NodeConstructor
//  TODO: | FunctionItemExpr
//  MapConstructor
//  TODO: | ArrayConstructor
//  TODO: | StringConstructor
//  TODO: | UnaryLookup
fn parse_primary_expr(input: &str) -> IResult<&str, Expr> {
    let result = parse_literal(input);
    if result.is_ok() {
        let (input, literal) = result?;
        return Ok((
            input,
            literal
        ))
    }

    let result = parse_function_call(input);
    if result.is_ok() {
        let (input, literal) = result?;
        return Ok((
            input,
            literal
        ))
    }

    let result = parse_map_constructor(input);
    if result.is_ok() {
        let (input, literal) = result?;
        return Ok((
            input,
            literal
        ))
    }

    result
}

// [137]    	FunctionCall 	   ::=    	EQName ArgumentList
fn parse_function_call(input: &str) -> IResult<&str, Expr> {

    println!("parse_function_call: {:?}", input);

    let (input, function) = parse_eqname(input)?;

    let (input, _) = tag("(")(input)?;

    let (input, arguments) = parse_arguments(input)?;

    let (input, _) = tag(")")(input)?;

    Ok((
        input,
        Expr::Call { function: Box::new(function), arguments }
    ))
}

// [138]    	Argument 	   ::=    	ExprSingle TODO: | ArgumentPlaceholder
fn parse_arguments(input: &str) -> IResult<&str, Vec<Statement>> {
    let mut arguments = vec![];

    let mut current_input = input;
    loop {
        println!("parse_arguments: {:?}", current_input);

        let (input, argument) = parse_expr_single(current_input)?;

        arguments.push(argument);

        let tmp = ws_tag(",", input);
        if tmp.is_err() {
            return
                Ok((
                    input,
                    arguments
                ))
        }
        current_input = tmp?.0;
    }
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

// [170]    	MapConstructor 	   ::=    	"map" "{" (MapConstructorEntry ("," MapConstructorEntry)*)? "}"
fn parse_map_constructor(input: &str) -> IResult<&str, Expr> {
    let input = ws_tag("map", input)?.0;

    let input = ws_tag("{", input)?.0;

    let mut entries = vec![];

    let mut current_input = input;
    loop {
        println!("parse_map_entries: {:?}", current_input);

        let (input, entry) = parse_map_constructor_entry(current_input)?;

        entries.push(entry);

        let input = ws(input)?.0;

        let tmp = tag(",")(input);
        if tmp.is_err() {

            let current_input = tag("}")(input)?.0;

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

// [218]    	EQName 	   ::=    	QName TODO: | URIQualifiedName
fn parse_eqname(input: &str) -> IResult<&str, Expr> {
    // [7]   	QName	   ::=   	PrefixedName | UnprefixedName
    // [8]   	PrefixedName	   ::=   	Prefix ':' LocalPart
    // [9]   	UnprefixedName	   ::=   	LocalPart
    // [10]   	Prefix	   ::=   	NCName
    // [11]   	LocalPart	   ::=   	NCName

    println!("parse_eqname: {:?}", input);

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
            Expr::QName { local_part: name2, url: String::from(url), prefix: name1 }
        ))
    } else {
        Ok((
            input,
            Expr::QName { local_part: name1, url: String::from(""), prefix: String::from("") } // TODO: resolve namespace
        ))
    }
}

// [4]   	NCName	   ::=   	Name - (Char* ':' Char*)	/* An XML Name, minus the ":" */
fn parse_ncname(input: &str) -> IResult<&str, String> {
    let (input, name_start) = take_while_m_n(0, 1, is_name_start_char)(input)?;
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

#[test]
fn simple_parser() {
    println!("{:?}", parse("xs:decimal(\"617375191608514839\") * xs:decimal(\"0\")"));
//    println!("{:?}", parse("let $a := 10, $b:=5 return $a"));
}