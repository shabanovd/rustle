use nom::IResult;
use crate::parser::errors::CustomError;
use crate::value::QName;
use crate::fns::Param;
use ordered_float::OrderedFloat;
use bigdecimal::BigDecimal;

const DEBUG: bool = false;

pub(crate) fn found_exprs(input: &str, exprs: Vec<Expr>) -> IResult<&str, Vec<Expr>, CustomError<&str>> {
    Ok((input, exprs))
}

pub(crate) fn found_expr(input: &str, expr: Expr) -> IResult<&str, Expr, CustomError<&str>> {
    if DEBUG {
        println!("\nfound: {:?}\ninput: {:?}", expr, input);
    }
    Ok((input, expr))
}

pub(crate) fn found_qname(input: &str, qname: QName) -> IResult<&str, QName, CustomError<&str>> {
    if DEBUG {
        println!("\nfound: {:?}\ninput: {:?}", qname, input);
    }
    Ok((input, qname))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Prolog(Vec<Expr>),
    Program(Expr),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Steps {
    One,
    More,
}

impl Steps {
    pub(crate) fn from(str: &str) -> Self {
        match str {
            "/" => Steps::One,
            "//" => Steps::More,
            _ => panic!("error")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Representation {
    Hexadecimal,
    Decimal
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    //internal
    Literals(Vec<Expr>),
    CharRef { representation: Representation, reference: u32 },
    EntityRef(String),
    EscapeQuot,
    EscapeApos,

    //prolog
    AnnotatedDecl { annotations: Vec<Expr>, decl: Box<Expr> },
    VarDecl { name: QName, type_declaration: Box<Option<Expr>>, external: bool, value: Box<Option<Expr>> },
    FunctionDecl { name: QName, params: Vec<Param>, type_declaration: Box<Option<Expr>>, external: bool, body: Option<Box<Expr>> },

    Body(Vec<Expr>),

    //navigation
    Root,
    Steps(Vec<Expr>),
    InitialPath { steps: Steps, expr: Box<Expr> },
    Path { steps: Steps, expr: Box<Expr> },
    AxisStep { step: Box<Expr>, predicates: Vec<Expr> },
    ForwardStep { attribute: bool, test: Box<Expr> },
    NameTest(QName),

    //spec
    Ident(String),

    Boolean(bool),
    Integer(i128),
    Decimal(BigDecimal),
    Double(OrderedFloat<f64>),
    StringComplex(Vec<Expr>),
    String(String),

    Item,

    ContextItem,

    Sequence(Box<Expr>),
    SequenceEmpty(),
    SequenceType { item_type: ItemType, occurrence_indicator: OccurrenceIndicator  },
    Range { from: Box<Expr>, till: Box<Expr> },
    Predicate(Box<Expr>),
//    Predicates(Vec<Statement>), // TODO: can it be covered by Sequence(Predicate)?

    Treat { expr: Box<Expr>, st: Box<Expr> },
    Castable { expr: Box<Expr>, st: Box<Expr> },
    Cast { expr: Box<Expr>, st: Box<Expr> },

    Postfix { primary: Box<Expr>, suffix: Vec<Expr> },

    Union(Vec<Expr>),
    IntersectExcept { left: Box<Expr>, is_intersect: bool, right: Box<Expr> },

    NodeDocument(Box<Expr>),
    NodeElement { name: Box<Expr>, attributes: Vec<Expr>, children: Vec<Expr> },
    NodeAttribute { name: Box<Expr>, value: Box<Expr> },
    NodeText(Box<Expr>),
    NodeComment(Box<Expr>),
    NodePI { target: Box<Expr>, content: Box<Expr> },

    Map { entries: Vec<Expr> }, // Expr because can't use MapEntry here
    MapEntry { key: Box<Expr>, value: Box<Expr> },

    SquareArrayConstructor(Vec<Expr>),
    CurlyArrayConstructor(Box<Expr>),

    QName { local_part: String, url: String, prefix: String },

    Unary { expr: Box<Expr>, sign_is_positive: bool },
    Binary { left: Box<Expr>, operator: OperatorArithmetic, right: Box<Expr> },
    Comparison { left: Box<Expr>, operator: OperatorComparison, right: Box<Expr> },

    If { condition: Box<Expr>, consequence: Box<Expr>, alternative: Box<Expr> },

    ArgumentList { arguments: Vec<Expr> },
    Function { arguments: Vec<Param>, body: Box<Expr> },
    Call { function: QName, arguments: Vec<Expr> },
    NamedFunctionRef { name: QName, arity: Box<Expr> },
    Annotation { name: QName, value: Option<String> },

    ParamList(Vec<Expr>),
    Param { name: QName, type_declaration: Box<Option<Expr>> },

    VarRef { name: QName },

    Or(Vec<Expr>),
    And(Vec<Expr>),
    StringConcat(Vec<Expr>),
    SimpleMap(Vec<Expr>),

    FLWOR { clauses: Vec<Expr>, return_expr: Box<Expr> },

    LetClause { bindings: Vec<Expr> },
    LetBinding { name: QName, type_declaration: Box<Option<Expr>>,  value: Box<Expr>},

    ForClause { bindings: Vec<Expr> },
    ForBinding { name: QName, values: Box<Expr>},

}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OperatorArithmetic {
    Plus,
    Minus,
    Multiply,
    Divide,
    IDivide,
    Mod,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OperatorComparison {
    GeneralEquals,
    GeneralNotEquals,
    GeneralLessThan,
    GeneralLessOrEquals,
    GeneralGreaterThan,
    GeneralGreaterOrEquals,

    ValueEquals,
    ValueNotEquals,
    ValueLessThan,
    ValueLessOrEquals,
    ValueGreaterThan,
    ValueGreaterOrEquals,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ItemType {
    Item,
    AtomicOrUnionType(QName)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OccurrenceIndicator {
    ExactlyOne,
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}