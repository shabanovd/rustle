use std::boxed::Box;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until, is_not},
    character::complete::{multispace0, multispace1}
};
use nom::character::complete::one_of;

use crate::namespaces::*;
use nom::lib::std::fmt::Error;
use nom::error::ParseError;

const DEBUG: bool = false;

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

    Sequence(Vec<Statement>),
    SequenceEmpty(),
    Range { from: Box<Expr>, till: Box<Expr> },
    Predicate(Box<Statement>),
//    Predicates(Vec<Statement>), // TODO: can it be covered by Sequence(Predicate)?

    Postfix { primary: Box<Expr>, suffix: Vec<Expr> },

    Node { name: Box<Expr>, attributes: Vec<Expr>, children: Vec<Expr> },
    Attribute { name: Box<Expr>, value: Box<Expr> },
    NodeText(String),
    NodeComment(String),
    NodePI { target: Box<Expr>, content: String },

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
    if DEBUG {
        println!("parse_expr_single: {:?}", input);
    }

    let (input, expr) = parse_range_expr(input)?;

    Ok((
        input,
        Statement::Expression( expr )
    ))
}


// [87]    	RangeExpr 	   ::=    	AdditiveExpr ( "to" AdditiveExpr )?
fn parse_range_expr(input: &str) -> IResult<&str, Expr> {
    let (input, from) = parse_binary_expr(input)?;

    let check = ws_tag("to", input);
    if check.is_ok() {
        let input = check?.0;

        let (input, till) = parse_binary_expr(input)?;

        Ok((
            input,
            Expr::Range { from: Box::new(from), till: Box::new(till) }
        ))
    } else {
        Ok((
            input,
            from
        ))
    }
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

//// [111]    	AxisStep 	   ::=    	(ReverseStep | ForwardStep) PredicateList
//// [123]    	PredicateList 	   ::=    	Predicate*
//fn parse_axis_step(input: &str) -> IResult<&str, Expr> {
//
//}

// [121]    	PostfixExpr 	   ::=    	TODO: PrimaryExpr (Predicate | ArgumentList | Lookup)*
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
        Ok((
            current_input,
            primary
        ))
    } else {
        Ok((
            current_input,
            Expr::Postfix { primary: Box::new(primary), suffix }
        ))
    }
}

// [124]    	Predicate 	   ::=    	"[" Expr "]"
fn parse_predicate(input: &str) -> IResult<&str, Expr> {
    let input = ws_tag("[", input)?.0;

    let (input, expr) = parse_expr_single(input)?;
//    let (input, expr) = parse_expr(input)?;

    let input = ws_tag("]", input)?.0;

    Ok((input, Expr::Predicate(Box::new(expr))))
}

// [128]    	PrimaryExpr 	   ::=    	Literal
//  TODO: | VarRef
//  | ParenthesizedExpr
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

    let result = parse_parenthesized_expr(input);
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

    let result = parse_node_constructor(input);
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
    if DEBUG {
        println!("parse_function_call: {:?}", input);
    }

    let (input, function) = parse_eqname(input)?;

    let input = tag("(")(input)?.0;

    let (input, arguments) = parse_arguments(input)?;

    let input = tag(")")(input)?.0;

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
        if DEBUG {
            println!("parse_arguments: {:?}", current_input);
        }

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
            Expr::NodePI { target: Box::new(target), content: String::from(content) }
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
        Expr::Node { name: Box::new( tag_name ), attributes, children }
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
                Expr::Attribute { name: Box::new(name), value: Box::new(Expr::String( String::from(value) )) }
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
// [36]    	EnclosedExpr 	   ::=    	"{" Expr? "}"

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

fn parse_qname(input: &str) -> IResult<&str, Expr> {
    // use as workaround
    parse_eqname(input)
}

// [218]    	EQName 	   ::=    	QName TODO: | URIQualifiedName
fn parse_eqname(input: &str) -> IResult<&str, Expr> {
    // [7]   	QName	   ::=   	PrefixedName | UnprefixedName
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