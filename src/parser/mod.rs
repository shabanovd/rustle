use std::boxed::Box;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until},
    character::complete::multispace0
};
use nom::character::complete::one_of;
use nom::Err::Error;

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

    QName { local_part: String, ns: String, prefix: String },

    Binary { left: Box<Expr>, operator: Operator, right: Box<Expr> },
    If { condition: Box<Expr>, consequence: Vec<Statement>, alternative: Vec<Statement> },

    Function { arguments: Vec<String>, body: Vec<Statement> },
    Call { function: Box<Expr>, arguments: Vec<Statement> },
//    Call { function: Box<Expr>, arguments: Vec<Statement> },

}

#[derive(Debug, PartialEq, Clone)]
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

        let (input, _) = multispace0(input)?;
        let tmp = tag(",")(input);
        if tmp.is_err() {
            return
                Ok((
                    input,
                    program
                ))
        }
        let (input, _) = tmp?;

        current_input = input;
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

    let (input, _) = multispace0(input)?;

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
            _ => panic!("parse error") // TODO: raise error instead
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

    let (input, _) = multispace0(input)?;

    let mut is_positive = true;
    let mut current_input = input;

    //TODO: optimize by relaxing
    loop {
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
//  TODO: | MapConstructor
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

    let (input, call) = parse_function_call(input)?;

    Ok((
        input,
        call
    ))
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
        println!("parse_arguments: {:?}", input);

        let (input, argument) = parse_expr_single(current_input)?;

        arguments.push(argument);

        let (input, _) = multispace0(input)?;
        let tmp = tag(",")(input);
        if tmp.is_err() {
            return
                Ok((
                    input,
                    arguments
                ))
        }
        let (input, _) = tmp?;

        current_input = input;
    }
}

// [129]    	Literal 	   ::=    	TODO: NumericLiteral | StringLiteral
fn parse_literal(input: &str) -> IResult<&str, Expr> {
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

// [222]    	StringLiteral 	   ::=    	('"' (PredefinedEntityRef | CharRef | EscapeQuot | [^"&])* '"') | ("'" (PredefinedEntityRef | CharRef | EscapeApos | [^'&])* "'")
fn parse_string_literal(input: &str) -> IResult<&str, Expr> {
    let (input, _) = tag("\"")(input)?;

    let (input, string) = take_until("\"")(input)?;

    let (input, _) = tag("\"")(input)?;

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

        Ok((
            input,
            Expr::QName { local_part: name2, ns: String::from(""), prefix: name1 } // TODO: resolve namespace
        ))
    } else {
        Ok((
            input,
            Expr::QName { local_part: name1, ns: String::from(""), prefix: String::from("") } // TODO: resolve namespace
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

#[test]
fn simple_parser() {
    println!("{:?}", parse("xs:decimal(\"617375191608514839\") * xs:decimal(\"0\")"));
//    println!("{:?}", parse("let $a := 10, $b:=5 return $a"));
}