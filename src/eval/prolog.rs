use crate::eval::expression::{Expression, NodeTest};
use crate::parser::op::{Representation, OperatorArithmetic, OperatorComparison};
use bigdecimal::BigDecimal;
use ordered_float::OrderedFloat;
use crate::values::{QName, resolve_function_qname, resolve_element_qname, Types, QNameResolved, atomization};
use crate::fns::{Param, call};
use crate::eval::{Environment, DynamicContext, EvalResult, Object, Type, eval_predicates, Axis, step_and_test, object_to_qname, object_owned_to_sequence, object_to_integer, ErrorInfo, INS, comparison};
use crate::serialization::{object_to_string};
use crate::serialization::to_string::object_to_string_xml;
use crate::eval::helpers::{relax, relax_sequences, sort_and_dedup, process_items, join_sequences};
use std::collections::HashMap;
use std::ops::ControlFlow;
use crate::eval::arithmetic::{eval_unary, eval_arithmetic};
use crate::eval::comparison::{eval_comparison, eval_comparison_item};
use crate::eval::piping::{Pipe, eval_pipe};
use crate::parser::errors::{CustomError, ErrorCode};
use crate::eval::sequence_type::{ItemType, OccurrenceIndicator, SequenceType, XS_ANY_ATOMIC_TYPE, XS_ANY_SIMPLE_TYPE, XS_NOTATION};
use linked_hash_map::LinkedHashMap;
use crate::namespaces::{Namespace, NS_heap};
use crate::eval::sequence_type::QNameToTypes;
use crate::parser::errors::ErrorCode::*;

//internal
#[derive(Clone, Debug)]
pub(crate) struct Literals { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for Literals {
    fn eval(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CharRef { pub(crate) representation: Representation, pub(crate) reference: u32 }

impl Expression for CharRef {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::CharRef { representation: self.representation.clone(), reference: self.reference.clone() }))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EntityRef {
    pub(crate) reference: String
}

impl EntityRef {
    pub(crate) fn boxed(name: &str) -> Box<dyn Expression> {
        Box::new(EntityRef { reference: String::from(name) })
    }
}

impl Expression for EntityRef {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::EntityRef(self.reference.clone())))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EscapeQuot {}

impl Expression for EscapeQuot {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::String(String::from("\"")))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EscapeApos {}

impl Expression for EscapeApos {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::String(String::from("'")))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VersionDecl {
    encoding: Option<String>,
    version: Option<String>,
}

impl VersionDecl {
    pub(crate) fn boxed<'a>(encoding: Option<String>, version: Option<String>) -> Result<Box<dyn Expression>, ErrorInfo> {
        if let Some(version) = &version {
            match version.as_str() {
                "1.0" | "3.0" | "3.1" => {
                    // TODO
                }
                _ => return Err((XQST0031, format!("unsupported version {}", version)))
            }
        }
        if let Some(encoding) = &encoding {
            match encoding.to_uppercase().replace("&#X2D;", "-").as_str() {
                "US-ASCII" |
                "ISO-8859-1" |
                "UTF-8" => {
                    // TODO
                },
                _ => return Err((XQST0087, format!("unsupported encoding {}", encoding)))
            }
        }
        Ok(Box::new(VersionDecl { encoding, version }))
    }
}

impl Expression for VersionDecl {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum BoundarySpace {
    Preserve,
    Strip
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareBoundarySpace {
    mode: BoundarySpace,
}

impl DeclareBoundarySpace {
    pub(crate) fn boxed(mode: BoundarySpace) -> Box<dyn Expression> {
        Box::new(DeclareBoundarySpace { mode })
    }
}

impl Expression for DeclareBoundarySpace {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.boundary_space.is_some() {
            Err((ErrorCode::XQST0068, String::from("TODO")))
        } else {
            env.boundary_space = Some(self.mode.clone());
            Ok((env, Object::Nothing))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareDefaultCollation {
    uri: String,
}

impl DeclareDefaultCollation {
    pub(crate) fn boxed(uri: String) -> Box<dyn Expression> {
        Box::new(DeclareDefaultCollation { uri })
    }
}

impl Expression for DeclareDefaultCollation {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.default_collation.is_some() {
            Err((ErrorCode::XQST0038, String::from("TODO")))
        } else {
            env.default_collation = Some(self.uri.clone());
            Ok((env, Object::Nothing))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareBaseURI {
    uri: Box<dyn Expression>
}

impl DeclareBaseURI {
    pub(crate) fn boxed(uri: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(DeclareBaseURI { uri })
    }
}

impl Expression for DeclareBaseURI {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (mut new_env, uri) = self.uri.eval(env, context)?;

        let uri = object_to_string(&new_env, &uri);

        new_env.static_base_uri = Some(uri);

        Ok((new_env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum ConstructionMode {
    Strip,
    Preserve
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareConstruction {
    mode: ConstructionMode,
}

impl DeclareConstruction {
    pub(crate) fn boxed(mode: ConstructionMode) -> Box<dyn Expression> {
        Box::new(DeclareConstruction { mode })
    }
}

impl Expression for DeclareConstruction {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.construction_mode.is_some() {
            Err((ErrorCode::XQST0067, String::from("TODO")))
        } else {
            env.construction_mode = Some(self.mode.clone());
            Ok((env, Object::Nothing))
        }
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum OrderingMode {
    Ordered,
    Unordered,
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareOrderingMode {
    mode: OrderingMode,
}

impl DeclareOrderingMode {
    pub(crate) fn boxed(mode: OrderingMode) -> Box<dyn Expression> {
        Box::new(DeclareOrderingMode { mode })
    }
}

impl Expression for DeclareOrderingMode {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.ordering_mode.is_some() {
            Err((ErrorCode::XQST0065, String::from("TODO")))
        } else {
            env.ordering_mode = Some(self.mode.clone());
            Ok((env, Object::Nothing))
        }
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum EmptyOrderMode {
    Greatest,
    Least,
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareEmptyOrder {
    mode: EmptyOrderMode,
}

impl DeclareEmptyOrder {
    pub(crate) fn boxed(mode: EmptyOrderMode) -> Box<dyn Expression> {
        Box::new(DeclareEmptyOrder { mode })
    }
}

impl Expression for DeclareEmptyOrder {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.empty_order_mode.is_some() {
            Err((ErrorCode::XQST0069, String::from("TODO")))
        } else {
            env.empty_order_mode = Some(self.mode.clone());
            Ok((env, Object::Nothing))
        }
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum PreserveMode {
    Preserve,
    NoPreserve
}

#[derive(Clone, Debug)]
pub enum InheritMode {
    Inherit,
    NoInherit
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareCopyNamespaces {
    preserve_mode: PreserveMode,
    inherit_mode: InheritMode,
}

impl DeclareCopyNamespaces {
    pub(crate) fn boxed(preserve_mode: PreserveMode, inherit_mode: InheritMode) -> Box<dyn Expression> {
        Box::new(DeclareCopyNamespaces { preserve_mode, inherit_mode })
    }
}

impl Expression for DeclareCopyNamespaces {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.copy_namespaces.is_some() {
            Err((ErrorCode::XQST0055, String::from("TODO")))
        } else {
            env.copy_namespaces = Some((self.preserve_mode.clone(), self.inherit_mode.clone()));
            Ok((env, Object::Nothing))
        }
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DecimalFormatPropertyName {
    DecimalSeparator,
    GroupingSeparator,
    Infinity,
    MinusSign,
    NaN,
    Percent,
    PerMille,
    ZeroDigit,
    Digit,
    PatternSeparator,
    ExponentSeparator
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareDecimalFormat {
    name: Option<QName>,
    properties: HashMap<DecimalFormatPropertyName, String>,
}

impl DeclareDecimalFormat {
    pub(crate) fn boxed(name: Option<QName>, properties: HashMap<DecimalFormatPropertyName, String>) -> Box<dyn Expression> {
        Box::new(DeclareDecimalFormat { name, properties })
    }
}

impl Expression for DeclareDecimalFormat {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if env.decimal_formats.is_none() {
            env.decimal_formats = Some(HashMap::new());
        };

        if let Some(map) = &mut env.decimal_formats {
            map.insert(self.name.clone(), self.properties.clone());
        };

        Ok((env, Object::Nothing))
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareNamespace {
    prefix: Box<dyn Expression>,
    uri: Box<dyn Expression>,
}

impl DeclareNamespace {
    pub(crate) fn boxed(prefix: Box<dyn Expression>, uri: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(DeclareNamespace { prefix, uri })
    }
}

impl Expression for DeclareNamespace {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, prefix) = self.prefix.eval(env, context)?;
        let prefix = object_to_string(&new_env, &prefix);

        let (mut new_env, uri) = self.uri.eval(new_env, context)?;
        let uri = object_to_string(&new_env, &uri);

        let ns = NS_heap { prefix, uri };

        new_env.namespaces.add(&ns);

        Ok((new_env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareDefaultNamespace {
    what: String,
    uri: Box<dyn Expression>
}

impl DeclareDefaultNamespace {
    pub(crate) fn boxed(what: &str, uri: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(DeclareDefaultNamespace { what: String::from(what), uri })
    }
}

impl Expression for DeclareDefaultNamespace {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (mut new_env, uri) = self.uri.eval(env, context)?;

        let uri = object_to_string(&new_env, &uri);

        match self.what.as_str() {
            "element" => {
                new_env.namespaces.default_for_element = Some(uri);
            }
            "function" => {
                new_env.namespaces.default_for_function = Some(uri);
            }
            _ => panic!("internal error")
        }

        Ok((new_env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

//prolog
#[derive(Clone, Debug)]
pub(crate) struct AnnotatedDecl {
    pub(crate) annotations: Vec<Box<dyn Expression>>,
    pub(crate) decl: Box<dyn Expression>
}

impl Expression for AnnotatedDecl {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        // TODO handle annotations
        self.decl.eval(env, context)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VarDecl {
    pub(crate) name: QName,
    pub(crate) type_declaration: Option<SequenceType>,
    pub(crate) external: bool,
    pub(crate) value: Option<Box<dyn Expression>>
}

impl Expression for VarDecl {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let name = resolve_element_qname(&self.name, &env);

        let new_env = (*env.clone()).next(); // TODO fix it

        if let Some(expr) = &self.value {
            match expr.eval(new_env, &DynamicContext::nothing()) {
                Ok((new_env, obj)) => {
                    env.set_variable(name, obj);
                },
                Err(e) => return Err(e),
            }
        }

        Ok((env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FunctionDecl {
    pub(crate) name: QName,
    pub(crate) params: Vec<Param>,
    pub(crate) st: Option<SequenceType>,
    pub(crate) external: bool,
    pub(crate) body: Option<Box<dyn Expression>>
}

impl Expression for FunctionDecl {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let name = resolve_function_qname(&self.name, &env);

        // TODO: handle typeDeclaration

        if let Some(body) = self.body.clone() {
            env.functions.put(name, self.params.clone(), self.st.clone(), body);

        } else {
            todo!()
        }

        Ok((env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeclareOption {
    pub(crate) name: QName,
    pub(crate) value: String,
}

impl DeclareOption {
    pub(crate) fn boxed(name: QName, value: String) -> Box<dyn Expression> {
        Box::new(DeclareOption { name, value })
    }
}

impl Expression for DeclareOption {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        env.set_option(self.name.clone(), self.value.clone());

        Ok((env, Object::Nothing))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Body {
    pub(crate) exprs: Vec<Box<dyn Expression>>
}

impl Body {
    pub(crate) fn empty() -> Box<dyn Expression> {
        Box::new(Body { exprs: vec![] })
    }

    pub(crate) fn new(exprs: Vec<Box<dyn Expression>>) -> Box<dyn Expression> {
        Box::new(Body { exprs })
    }
}

impl Expression for Body {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if self.exprs.len() == 0 {
            Ok((env, Object::Empty))
        } else {
            let mut current_env = env;

            let mut evaluated = vec![];
            for expr in &self.exprs {
                let (new_env, value) = expr.eval(current_env, context)?;
                current_env = new_env;

                match value {
                    Object::Empty => {},
                    _ => evaluated.push(value)
                }
            }
            // TODO understand when it should happen... sort_and_dedup(&mut evaluated);
            relax(current_env, evaluated)
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EnclosedExpr {
    pub(crate) expr: Box<dyn Expression>
}

impl EnclosedExpr {
    pub(crate) fn new(expr: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(EnclosedExpr { expr })
    }
}

impl Expression for EnclosedExpr {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let new_env = env.next();
        let (new_env, value) = self.expr.eval(new_env, context)?;
        env = new_env.prev();

        Ok((env, value))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Pragma {
    pub(crate) name: QName,
    pub(crate) content: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ExtensionExpr {
    pragma: Vec<Pragma>,
    expr: Option<Box<dyn Expression>>
}

impl ExtensionExpr {
    pub(crate) fn boxed(pragma: Vec<Pragma>, expr: Option<Box<dyn Expression>>) -> Box<dyn Expression> {
        Box::new(ExtensionExpr { pragma, expr })
    }
}

impl Expression for ExtensionExpr {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        // TODO handle pragmas
        if let Some(expr) = self.expr.as_ref() {
            let new_env = env.next();
            let (new_env, value) = expr.eval(new_env, context)?;
            env = new_env.prev();

            Ok((env, value))
        } else {
            Ok((env, Object::Empty))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ValidationMode {
    Lax,
    Strict,
    Type(QName),
}

impl From<&str> for ValidationMode {
    fn from(name: &str) -> Self {
        match name {
            "lax" => ValidationMode::Lax,
            "strict" => ValidationMode::Strict,
            _ => panic!("internal error")
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidateExpr {
    pub(crate) mode: Option<ValidationMode>,
    expr: Box<dyn Expression>
}

impl ValidateExpr {
    pub(crate) fn boxed(mode: Option<ValidationMode>, expr:Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(ValidateExpr { mode, expr })
    }
}

impl Expression for ValidateExpr {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, node) = self.expr.eval(env, context)?;

        match &self.mode {
            Some(mode) => {
                match mode {
                    ValidationMode::Lax => todo!(),
                    ValidationMode::Strict => todo!(),
                    ValidationMode::Type(name) => todo!(),
                }
            }
            None => todo!()
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

//navigation
#[derive(Clone, Debug)]
pub(crate) struct Root {}

impl Expression for Root {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        match &context.item {
            Object::Empty => Ok((env, Object::Empty)),
            Object::Node(rf) => {
                if let Some(node) = rf.root() {
                    Ok((env, Object::Node(node)))
                } else {
                    Err((ErrorCode::TODO, String::from("TODO")))
                }
            },
            _ => Err((ErrorCode::TODO, String::from("TODO")))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Steps {
    pub(crate) steps: Vec<Box<dyn Expression>>
}

impl Steps {
    pub(crate) fn new(steps: Vec<Box<dyn Expression>>) -> Box<Self> {
        Box::new(Steps { steps })
    }
}

impl Expression for Steps {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;
        let mut current_context = context.clone();
        for step in &self.steps {
            let (new_env, value) = step.eval(current_env, &current_context)?;
            current_env = new_env;

            current_context = DynamicContext {
                initial_node_sequence: None,
                item: value,
                position: None,
                last: None,
            };
        }

        Ok((current_env, current_context.item))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct InitialPath {
    pub(crate) initial_node_sequence: INS,
    pub(crate) expr: Box<dyn Expression>
}

impl Expression for InitialPath {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        // TODO: handle steps
        // "/"  (fn:root(self::node()) treat as document-node())/
        // "//" (fn:root(self::node()) treat as document-node())/descendant-or-self::node()/

        let context = DynamicContext {
            initial_node_sequence: Some(self.initial_node_sequence.clone()),
            item: context.item.clone(),
            position: None,
            last: None,
        };

        self.expr.eval(env, &context)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Path {
    pub(crate) initial_node_sequence: Option<INS>,
    pub(crate) expr: Box<dyn Expression>
}

impl Expression for Path {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        // TODO: optimize
        let context = DynamicContext {
            initial_node_sequence: self.initial_node_sequence.clone(),
            item: context.item.clone(),
            position: context.position.clone(),
            last: context.last.clone(),
        };

        self.expr.eval(env, &context)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AxisStep {
    pub(crate) step: Box<dyn Expression>,
    pub(crate) predicates: Vec<PrimaryExprSuffix>
}

impl AxisStep {
    pub(crate) fn boxed(step: Box<dyn Expression>, predicates: Vec<PrimaryExprSuffix>) -> Box<dyn Expression> {
        Box::new(AxisStep { step, predicates })
    }
}

impl Expression for AxisStep {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, value) = self.step.eval(current_env, context)?;
        current_env = new_env;

        eval_predicates(&self.predicates, current_env, value, context)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ForwardStep { pub(crate) axis: Axis, pub(crate) test: Box<dyn NodeTest> }

impl Expression for ForwardStep {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (env, result) = step_and_test(&self.axis, &self.test, env, context)?;
        // println!("ForwardStep: {:?}", result);
        Ok((env, result))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

//spec
#[derive(Clone, Debug)]
pub(crate) struct Ident { pub(crate) value: String }

impl Expression for Ident {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Boolean { pub(crate) bool: bool }

impl Expression for Boolean {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::Boolean(self.bool))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Integer { pub(crate) number: i128 }

impl Expression for Integer {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::Integer(self.number))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        let pos = self.number;
        if pos <= 0 {
            Ok((env, Object::Empty))
        } else {
            match value {
                Object::Empty => Ok((env, Object::Empty)),
                Object::Atomic(..) |
                Object::Node {..} => {
                    if pos == 1 {
                        Ok((env, value))
                    } else {
                        Ok((env, Object::Empty))
                    }
                }
                Object::Range { min, max } => {
                    let len = max - min + 1;

                    if pos > len {
                        Ok((env, Object::Empty))
                    } else {
                        let num = min + pos - 1;
                        Ok((env, Object::Atomic(Type::Integer(num))))
                    }
                },
                Object::Array(items) |
                Object::Sequence(items) => {
                    if let Some(item) = items.get((pos - 1) as usize) {
                        Ok((env, item.clone()))
                    } else {
                        Ok((env, Object::Empty))
                    }
                },
                _ => panic!("predicate {:?} on {:?}", pos, value)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Decimal { pub(crate) number: BigDecimal }

impl Expression for Decimal {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::Decimal(self.number.clone()))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Double { pub(crate) number: OrderedFloat<f64> }

impl Expression for Double {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::Double(self.number))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringComplex {
    pub(crate) exprs: Vec<Box<dyn Expression>>
}

impl StringComplex {
    pub(crate) fn boxed(exprs: Vec<Box<dyn Expression>>) -> Box<dyn Expression> {
        Box::new(StringComplex { exprs })
    }
}

impl Expression for StringComplex {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let mut strings = Vec::with_capacity(self.exprs.len());
        for expr in &self.exprs {
            let (new_env, object) = expr.eval(current_env, context)?;
            current_env = new_env;

            let str = object.to_string()?;
            strings.push(str);
        }

        Ok((current_env, Object::Atomic(Type::String(strings.join("")))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringExpr {
    pub(crate) string: String
}

impl StringExpr {
    pub(crate) fn empty() -> Box<dyn Expression> {
        Box::new(StringExpr { string: String::new() })
    }

    pub(crate) fn new(string: String) -> Box<dyn Expression> {
        Box::new(StringExpr { string })
    }
}

impl From<&str> for StringExpr {
    fn from(value: &str) -> Self {
        StringExpr { string: String::from(value) }
    }
}

impl From<String> for StringExpr {
    fn from(string: String) -> Self {
        StringExpr { string }
    }
}

impl Expression for StringExpr {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic(Type::String(self.string.clone()))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Item {}

impl Expression for Item {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContextItem {}

impl Expression for ContextItem {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, context.item.clone()))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Sequence { pub(crate) expr: Box<dyn Expression> }

impl Sequence {
    pub(crate) fn empty() -> Box<dyn Expression> {
        Box::new(SequenceEmpty {})
    }

    pub(crate) fn new(expr: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(Sequence { expr })
    }
}

impl Expression for Sequence {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, value) = self.expr.eval(env, context)?;

        let mut items = object_owned_to_sequence(value);
        let mut result= Vec::with_capacity(items.len());
        relax_sequences(&mut result, items);
        relax(new_env, result)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SequenceEmpty {}

impl Expression for SequenceEmpty {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Empty))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Range { pub(crate) from: Box<dyn Expression>, pub(crate) till: Box<dyn Expression> }

impl Expression for Range {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated_from) = self.from.eval(current_env, context)?;
        current_env = new_env;

        match evaluated_from {
            Object::Empty => return Ok((current_env, Object::Empty)),
            _ => {}
        }

        let (new_env, evaluated_till) = self.till.eval(current_env, context)?;
        current_env = new_env;

        match evaluated_till {
            Object::Empty => return Ok((current_env, Object::Empty)),
            _ => {}
        }

        let min = match object_to_integer(&current_env, evaluated_from) {
            Ok(num) => num,
            Err(e) => return Err(e)
        };

        let max = match object_to_integer(&current_env, evaluated_till) {
            Ok(num) => num,
            Err(e) => return Err(e)
        };

        if min > max {
            Ok((current_env, Object::Empty))
        } else if min == max {
            Ok((current_env, Object::Atomic(Type::Integer(min))))
        } else {
            Ok((current_env, Object::Range { min, max }))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PrimaryExprSuffix {
    pub(crate) predicate: Option<Box<dyn Expression>>,
    pub(crate) argument_list: Option<Vec<Box<dyn Expression>>>,
    pub(crate) lookup: Option<Box<dyn Expression>>,
}

#[derive(Clone, Debug)]
pub(crate) struct InstanceOf {
    pub(crate) expr: Box<dyn Expression>,
    pub(crate) st: SequenceType
}

impl Expression for InstanceOf {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, object) = self.expr.eval(current_env, context)?;
        current_env = new_env;

        // TODO occurrence_indicator checks

        let result = self.st.is_castable(&current_env, &object)?;
        Ok((current_env, Object::Atomic(Type::Boolean(result))))

        // process_items(current_env, object, |env, item| {
        //     let result = self.st.is_castable(&env, &item)?;
        //     Ok((env, Object::Atomic(Type::Boolean(result))))
        // })
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Treat {
    pub(crate) expr: Box<dyn Expression>,
    pub(crate) st: SequenceType
}

impl Expression for Treat {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, object) = self.expr.eval(current_env, context)?;
        current_env = new_env;

        // TODO occurrence_indicator checks

        process_items(current_env, object, |env, item, position, last| {
            let correct = self.st.is_castable(&env, &item)?;

            if correct {
                Ok((env, item))
            } else {
                Err((ErrorCode::XPDY0050, String::from("TODO")))
            }
        })
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Castable { pub(crate) expr: Box<dyn Expression>, pub(crate) st: SequenceType }

impl Expression for Castable {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, object) = self.expr.eval(env, context)?;

        let v = self.st.is_castable(&new_env, &object)?;

        Ok((new_env, Object::Atomic(Type::Boolean(v))))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        process_items(env, value, |env, item, position, last| {

            let current_context = DynamicContext {
                initial_node_sequence: None,
                item: item.clone(), position: Some(position), last
            };

            let (new_env, object) = self.expr.eval(env, &current_context)?;

            if self.st.is_castable(&new_env, &object)? {
                Ok((new_env, item))
            } else {
                Ok((new_env, Object::Nothing))
            }
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Cast { pub(crate) expr: Box<dyn Expression>, pub(crate) st: SequenceType }

impl Expression for Cast {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, object) = self.expr.eval(env, context)?;

        match object {
            Object::Empty => {
                if self.st.occurrence_indicator == OccurrenceIndicator::ZeroOrOne {
                    Ok((new_env, Object::Empty))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Object::Atomic(t) => {
                match &self.st.item_type {
                    ItemType::AtomicOrUnionType(name) => {
                        let name: QNameResolved = new_env.namespaces.resolve(&name);
                        if name == XS_NOTATION || name == XS_ANY_SIMPLE_TYPE || name == XS_ANY_ATOMIC_TYPE {
                            Err((ErrorCode::XPST0080, String::from("TODO")))
                        } else if let Some(types) = QNameToTypes.get(&name) {
                            Ok((new_env, Object::Atomic(t.convert(types.clone())?)))
                        } else {
                            // TODO custom types
                            Err((ErrorCode::XQST0052, String::from("TODO")))
                        }
                    }
                    _ => panic!("raise error?")
                }
            }
            _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Postfix { pub(crate) primary: Box<dyn Expression>, pub(crate) suffix: Vec<PrimaryExprSuffix> }

impl Expression for Postfix {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, value) = self.primary.eval(env, context)?;

        eval_predicates(&self.suffix, new_env, value, context)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Union { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for Union {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let mut result = Vec::with_capacity(self.exprs.len());
        for expr in &self.exprs {
            let (new_env, items) = expr.eval(current_env, context)?;
            current_env = new_env;

            let items = object_owned_to_sequence(items);

            join_sequences(&mut result, items);
            sort_and_dedup(&mut result)
        }

        relax(current_env, result)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IntersectExcept { pub(crate) left: Box<dyn Expression>, pub(crate) is_intersect: bool, pub(crate) right: Box<dyn Expression> }

impl Expression for IntersectExcept {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeDocument {
    pub(crate) expr: Box<dyn Expression>
}

impl NodeDocument {
    pub(crate) fn new(expr: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodeDocument { expr })
    }
}

impl Expression for NodeDocument {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env.next();
        let rf = current_env.xml_writer(|w| w.start_document());

        let (new_env, obj) = self.expr.eval(current_env, context)?;
        current_env = new_env;

        match obj {
            Object::Empty => {},
            Object::Sequence(items) => {
                for item in items {
                    match item {
                        Object::Node(rf) => {
                            if current_env.xml_tree_id() != rf.xml_tree_id() {
                                current_env.xml_writer(|w| w.link_node(&rf));
                            }
                        },
                        _ => panic!("unexpected object {:?}", item) //TODO: better error
                    }
                }
            },
            Object::Node(rf) => {
                if current_env.xml_tree_id() != rf.xml_tree_id() {
                    current_env.xml_writer(|w| w.link_node(&rf));
                }
            },
            _ => panic!("unexpected object {:?}", obj) //TODO: better error
        };

        current_env.xml_writer(|w| w.start_document());

        current_env = current_env.prev();

        Ok((current_env, Object::Node(rf)))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Attributes {
    pub(crate) pairs: LinkedHashMap<QName, Box<dyn Expression>>,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            pairs: LinkedHashMap::new()
        }
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn add(&mut self, name: QName, value: Box<dyn Expression>) -> Result<(), ErrorInfo> {
        if let Some(..) = self.pairs.insert(name, value) {
            Err((ErrorCode::XQST0040, String::from("TODO")))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeElement {
    pub(crate) name: Box<dyn Expression>,
    pub(crate) attributes: Option<Attributes>,
    pub(crate) children: Vec<Box<dyn Expression>>
}

impl NodeElement {
    pub(crate) fn new(
        name: Box<dyn Expression>,
        attributes: Option<Attributes>,
        children: Vec<Box<dyn Expression>>
    ) -> Box<dyn Expression> {
        Box::new(NodeElement { name, attributes, children })
    }

    fn process_items(&self, env: &mut Box<Environment>, object: Object) {
        let elements = vec![];
        let elements = self.processing(env, object, elements);

        let content = elements.join(" ");
        env.xml_writer(|w| w.text(content));
    }

    fn processing(&self, env: &mut Box<Environment>, object: Object, elements: Vec<String>) -> Vec<String> {
        let mut elements = elements;
        match object {
            Object::Empty => {},
            Object::Array(items) |
            Object::Sequence(items) => {
                for item in items {
                    elements = self.processing(env, item, elements);
                }
            },
            Object::Node(rf) => {
                let content = elements.join(" ");
                env.xml_writer(|w| w.text(content));

                if env.xml_tree_id() != rf.xml_tree_id() {
                    env.xml_writer(|w| w.link_node(&rf));
                }
            },
            Object::Atomic(..) => {
                let content = object_to_string(env, &object);
                elements.push(content);
            }
            _ => panic!("unexpected object {:?}", object) //TODO: better error
        };

        elements
    }
}

impl Expression for NodeElement {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated_name) = self.name.eval(current_env, context)?;
        current_env = new_env;

        let name = object_to_qname(evaluated_name);

        let rf = current_env.xml_writer(|w| w.start_element(name));

        if let Some(attributes) = &self.attributes {
            for attribute in &attributes.pairs {
                let (new_env, evaluated_value) = attribute.1.eval(current_env, context)?;
                current_env = new_env;

                let value = object_to_string(&current_env, &evaluated_value);

                current_env.xml_writer(|w| w.attribute(attribute.0.clone(), value));
            }
        }

        for child in &self.children {
            let (new_env, evaluated) = child.eval(current_env, context)?;
            current_env = new_env;

            self.process_items(&mut current_env, evaluated)
        }

        current_env.xml_writer(|w| w.end_element().unwrap()); // TODO check?
        Ok((current_env, Object::Node(rf) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeAttribute { pub(crate) name: Box<dyn Expression>, pub(crate) value: Box<dyn Expression> }

impl NodeAttribute {
    pub(crate) fn new(name: Box<dyn Expression>, value: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodeAttribute { name, value })
    }
}

impl Expression for NodeAttribute {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated_name) = self.name.eval(current_env, context)?;
        current_env = new_env;

        let name = object_to_qname(evaluated_name);

        let (new_env, evaluated_value) = self.value.eval(current_env, context)?;
        current_env = new_env;

        let value = object_to_string(&current_env, &evaluated_value);

        let rf = current_env.xml_writer(|w| w.attribute(name, value));

        Ok((current_env, Object::Node(rf) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeText {
    pub(crate) content: Box<dyn Expression>
}

impl NodeText {
    pub(crate) fn new(content: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodeText { content })
    }
}

impl Expression for NodeText {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (mut new_env, evaluated) = self.content.eval(env, context)?;

        let content = object_to_string(&new_env, &evaluated);

        let pointer = new_env.xml_writer(|w| w.text(content));

        Ok((new_env, Object::Node(pointer) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeComment {
    pub(crate) content: Box<dyn Expression>
}
impl NodeComment {
    pub(crate) fn new(content: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodeComment { content })
    }
}

impl From<&str> for NodeComment {
    fn from(content: &str) -> Self {
        NodeComment { content: Box::new(StringExpr::from(content)) }
    }
}

impl Expression for NodeComment {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (mut new_env, evaluated) = self.content.eval(env, context)?;

        let content = object_to_string(&new_env, &evaluated);

        let rf = new_env.xml_writer(|w| w.comment(content));

        Ok((new_env, Object::Node(rf) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodePI {
    pub(crate) target: Box<dyn Expression>,
    pub(crate) content: Box<dyn Expression>
}

impl NodePI {
    pub(crate) fn new(target: Box<dyn Expression>, content: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodePI { target, content })
    }
}

impl Expression for NodePI {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env= env;

        let (new_env, evaluated_target) = self.target.eval(current_env, context)?;
        current_env = new_env;

        let target = object_to_qname(evaluated_target);

        let (new_env, evaluated) = self.content.eval(current_env, context)?;
        current_env = new_env;

        let content = object_to_string(&current_env, &evaluated);

        let rf = current_env.xml_writer(|w| w.pi(target, content));

        Ok(( current_env, Object::Node(rf) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeNS {
    pub(crate) prefix: Box<dyn Expression>,
    pub(crate) url: Box<dyn Expression>
}

impl NodeNS {
    pub(crate) fn boxed(prefix: Box<dyn Expression>, url: Box<dyn Expression>) -> Box<dyn Expression> {
        Box::new(NodeNS { prefix, url })
    }
}

impl Expression for NodeNS {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env= env;

        let (new_env, prefix) = self.prefix.eval(current_env, context)?;
        current_env = new_env;

        let prefix = object_to_string(&current_env, &prefix);

        let (new_env, url) = self.url.eval(current_env, context)?;
        current_env = new_env;

        let url = object_to_string(&current_env, &url);

        let rf = current_env.xml_writer(|w| w.ns(prefix, url));

        Ok(( current_env, Object::Node(rf) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Map { pub(crate) entries: Vec<MapEntry> } // Expr because can't use MapEntry here

impl Expression for Map {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let mut map = HashMap::new();
        for MapEntry { key, value } in &self.entries {

            let (new_env, evaluated_key) = key.eval(current_env, context)?;
            current_env = new_env;

            let (new_env, evaluated_value) = value.eval(current_env, context)?;
            current_env = new_env;

            match evaluated_key {
                Object::Atomic(key_object) => {
                    map.insert(key_object, evaluated_value);
                }
                _ => panic!("wrong expression") //TODO: proper code
            }
        }

        Ok((current_env, Object::Map(map)))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MapEntry { pub(crate) key: Box<dyn Expression>, pub(crate) value: Box<dyn Expression> }

impl Expression for MapEntry {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SquareArrayConstructor { pub(crate) items: Vec<Box<dyn Expression>> }

impl Expression for SquareArrayConstructor {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let mut values = Vec::with_capacity(self.items.len());
        for item in &self.items {
            let (new_env, evaluated) = item.eval(current_env, context)?;
            current_env = new_env;

            values.push(evaluated);
        }

        Ok((current_env, Object::Array(values)))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CurlyArrayConstructor { pub(crate) expr: Box<dyn Expression> }

impl Expression for CurlyArrayConstructor {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let (new_env, evaluated) = self.expr.eval(env, context)?;

        let values = match evaluated {
            Object::Empty => vec![],
            Object::Array(items) |
            Object::Sequence(items) => items,
            Object::Range {..} |
            Object::Node(_) |
            Object::Atomic(_) => {
                let mut items = Vec::with_capacity(1);
                items.push(evaluated);
                items
            }
            _ => panic!("can't convert to array {:?}", evaluated)
        };

        Ok((new_env, Object::Array(values)))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Unary { pub(crate) expr: Box<dyn Expression>, pub(crate) sign_is_positive: bool }

impl Expression for Unary {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated) = self.expr.eval(current_env, context)?;
        current_env = new_env;

        process_items(current_env, evaluated, |env, item, position, last| {
            match item {
                Object::Empty => Ok((env, Object::Empty)),
                _ => eval_unary(env, item, self.sign_is_positive)
            }
        })
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Binary { pub(crate) left: Box<dyn Expression>, pub(crate) operator: OperatorArithmetic, pub(crate) right: Box<dyn Expression> }

impl Expression for Binary {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, left_result) = self.left.eval(current_env, context)?;
        current_env = new_env;

        if left_result == Object::Empty {
            Ok((current_env, Object::Empty))
        } else {
            let (new_env, right_result) = self.right.eval(current_env, context)?;
            current_env = new_env;

            eval_arithmetic(current_env, self.operator.clone(), left_result, right_result)
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Comparison {
    pub(crate) left: Box<dyn Expression>,
    pub(crate) operator: OperatorComparison,
    pub(crate) right: Box<dyn Expression>
}

impl Expression for Comparison {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, left_result) = self.left.eval(current_env, context)?;
        current_env = new_env;

        let (new_env, right_result) = self.right.eval(current_env, context)?;
        current_env = new_env;

        eval_comparison(current_env, self.operator.clone(), left_result, right_result)
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        let mut current_env = env;

        let (it, total) = value.into_iter_with_total();

        let mut evaluated = Vec::with_capacity(total);

        let mut position = 0;
        for item in it {
            position += 1;
            let context = DynamicContext {
                initial_node_sequence: None,
                item, position: Some(position), last: Some(total),
            };

            let (new_env, l_value) = self.left.eval(current_env, &context)?;
            current_env = new_env;

            let (new_env, r_value) = self.right.eval(current_env, &context)?;
            current_env = new_env;

            println!("l_value: {:?}", l_value);
            println!("r_value: {:?}", r_value);

            let (new_env, v) = eval_comparison_item(
                current_env, self.operator.clone(), l_value, r_value
            )?;
            current_env = new_env;

            if v {
                evaluated.push(context.item)
            }
        }

        relax(current_env, evaluated)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct If {
    pub(crate) condition: Box<dyn Expression>,
    pub(crate) consequence: Box<dyn Expression>,
    pub(crate) alternative: Box<dyn Expression>
}

impl Expression for If {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated) = self.condition.eval(current_env, context)?;
        current_env = new_env;

        let (new_env, evaluated) = if evaluated.effective_boolean_value()? {
            self.consequence.eval(current_env, context)?
        } else {
            self.alternative.eval(current_env, context)?
        };
        Ok((new_env, evaluated))
    }

    fn predicate<'a>(&self, env: Box<Environment>, _context: &DynamicContext, value: Object) -> EvalResult {
        process_items(env, value, |env, item, position, last| {

            let current_context = DynamicContext {
                initial_node_sequence: None,
                item: item.clone(),
                position: Some(position),
                last
            };

            let (new_env, evaluated) = self.condition.eval(env, &current_context)?;

            let (new_env, evaluated) = if evaluated.effective_boolean_value()? {
                self.consequence.eval(new_env, &current_context)?
            } else {
                self.alternative.eval(new_env, &current_context)?
            };

            if evaluated.effective_boolean_value()? {
                Ok((new_env, item))
            } else {
                Ok((new_env, Object::Nothing))
            }
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Function {
    pub(crate) arguments: Vec<Param>,
    pub(crate) st: Option<SequenceType>,
    pub(crate) body: Box<dyn Expression>
}

impl Expression for Function {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Function { parameters: self.arguments.clone(), st: self.st.clone(), body: self.body.clone() }))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Call {
    pub(crate) function: QName,
    pub(crate) arguments: Vec<Box<dyn Expression>>
}

impl Expression for Call {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let name = resolve_function_qname(&self.function, &current_env);

        let (parameters, body) = match current_env.get_variable(&name) {
            Some(Object::Function {parameters, st, body}) => (parameters, body),
            None => {
                let mut evaluated_arguments = Vec::with_capacity(self.arguments.len());
                for argument in &self.arguments {
                    let (new_env, value) = argument.eval(current_env, context)?;
                    current_env = new_env;

                    evaluated_arguments.push(value);
                }

                return call(current_env, name, evaluated_arguments, context);
            }
            _ => panic!("error")
        };

        assert_eq!(parameters.len(), self.arguments.len(), "wrong number of parameters");

        let mut arguments = Vec::with_capacity(parameters.len());

        for (parameter, argument) in parameters.into_iter().zip(self.arguments.clone().into_iter()) {
            let (new_env, new_result) = argument.eval(current_env, context)?;
            current_env = new_env;

            let name = resolve_function_qname(&parameter.name, &current_env);

            arguments.push((name, new_result));
        }

        let mut fn_env = current_env.next();
        for (name, value) in arguments {
            fn_env.set_variable(name, value);
        }

        let (new_env, result) = body.eval(fn_env, context)?;
        current_env = new_env.prev();

        Ok((current_env, result))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        let mut current_env = env;

        let items = object_owned_to_sequence(value);

        let mut evaluated = vec![];

        let last = Some(items.len());
        let mut position = 0;
        for item in items {
            position += 1;
            let context = DynamicContext {
                initial_node_sequence: None,
                item, position: Some(position), last
            };

            let (new_env, result) = self.eval(current_env, &context)?;
            current_env = new_env;

            if result.effective_boolean_value()? {
                evaluated.push(context.item)
            }
        }

        relax(current_env, evaluated)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ArgumentPlaceholder {
}

impl ArgumentPlaceholder {
    pub(crate) fn boxed() -> Box<dyn Expression> {
        Box::new(ArgumentPlaceholder {})
    }
}

impl Expression for ArgumentPlaceholder {
    fn eval<'a>(&self, env: Box<Environment>, _context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Placeholder))
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NamedFunctionRef {
    pub(crate) name: QName,
    pub(crate) arity: Box<dyn Expression>
}

impl Expression for NamedFunctionRef {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, arity) = self.arity.eval(current_env, context)?;
        current_env = new_env;

        let arity = match object_to_integer(&current_env, arity) {
            Ok(num) => {
                if num > 0 {
                    num as usize
                } else {
                    return Err((ErrorCode::TODO, String::from("TODO")))
                }
            },
            Err(e) => return Err(e)
        };

        let name = resolve_function_qname(&self.name, &current_env);

        Ok((current_env, Object::FunctionRef { name, arity }))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Annotation {
    pub(crate) name: QName,
    pub(crate) value: Option<String>
}

impl Expression for Annotation {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VarRef { pub(crate) name: QName }

impl Expression for VarRef {
    fn eval<'a>(&self, env: Box<Environment>, _context: &DynamicContext) -> EvalResult {
        let name = resolve_element_qname(&self.name, &env);

        if let Some(value) = env.get_variable(&name) {
            Ok((env, value))
        } else {
            Err((ErrorCode::XPST0008, format!("unknown variable {:?}", name)))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Or { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for Or {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        if self.exprs.len() == 0 {
            Ok((env, Object::Empty))
        } else {
            let mut current_env = env;

            let mut sequence = Vec::with_capacity(self.exprs.len());
            for expr in &self.exprs {
                let (new_env, evaluated) = expr.eval(current_env, context)?;
                current_env = new_env;

                sequence.push(evaluated);
            }

            if sequence.len() == 0 {
                Ok((current_env, Object::Empty))
            } else if sequence.len() == 1 {
                let object = sequence.remove(0);
                Ok((current_env, object))
            } else {
                let mut value = None;
                let mut v;
                for item in sequence {
                    v = item.effective_boolean_value()?;

                    if let Some(acc) = value {
                        value = Some(acc || v);
                    } else {
                        value = Some(v);
                    }

                    if value == Some(true) {
                        break
                    }
                }

                if let Some(acc) = value {
                    Ok((current_env, Object::Atomic(Type::Boolean(acc))))
                } else {
                    panic!("internal error")
                }
            }
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct And { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for And {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let result = if self.exprs.len() == 0 {
            Object::Empty
        } else {
            let mut sequence = Vec::with_capacity(self.exprs.len());
            for expr in &self.exprs {
                let (new_env, evaluated) = expr.eval(current_env, context)?;
                current_env = new_env;

                sequence.push(evaluated);
            }

            let result: Object = if sequence.len() == 0 {
                Object::Empty
            } else if sequence.len() == 1 {
                sequence.remove(0)
            } else {
                let mut value = None;
                let mut v;
                for item in sequence {
                    v = item.effective_boolean_value()?;

                    println!("AND {:?} -> {:?}", item, v);

                    if let Some(acc) = value {
                        if !v {
                            value = Some(false);
                            break;
                        }
                    } else {
                        value = Some(v);
                        if !v {
                            break;
                        }
                    }
                }

                if let Some(acc) = value {
                    Object::Atomic(Type::Boolean(acc))
                } else {
                    panic!("internal error")
                }
            };
            result
        };
        Ok((current_env, result))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringConcat { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for StringConcat {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        if self.exprs.len() == 0 {
            Ok((current_env, Object::Atomic(Type::String(String::new()))))
        } else {
            let mut sequence = Vec::with_capacity(self.exprs.len());
            for expr in &self.exprs {
                let (new_env, evaluated) = expr.eval(current_env, context)?;
                current_env = new_env;

                sequence.push(evaluated);
            }

            if sequence.len() == 0 {
                Ok((current_env, Object::Atomic(Type::String(String::new()))))
            } else if sequence.len() == 1 {
                let object = sequence.remove(0);
                Ok((current_env, object))
            } else {
                let str = sequence.into_iter()
                    .map(|item| object_to_string(&current_env, &item))
                    .collect();

                Ok((current_env, Object::Atomic(Type::String(str))))
            }
        }
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SimpleMap { pub(crate) exprs: Vec<Box<dyn Expression>> }

impl Expression for SimpleMap {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let mut result = Object::Empty;
        let mut i = 0;
        for expr in &self.exprs {
            if i == 0 {
                let (new_env, evaluated) = expr.eval(current_env, context)?;
                current_env = new_env;

                result = evaluated;
            } else {
                let mut sequence = vec![];

                let it = object_owned_to_sequence(result);
                let last = Some(it.len());
                let mut position = 0;
                for item in it {
                    position += 1;
                    let current_context = DynamicContext {
                        initial_node_sequence: None,
                        item, position: Some(position), last
                    };
                    let (new_env, evaluated) = expr.eval(current_env, &current_context)?;
                    current_env = new_env;

                    println!("evaluated: {:?}", evaluated);

                    let items = object_owned_to_sequence(evaluated);
                    relax_sequences(&mut sequence, items);
                }
                sort_and_dedup(&mut sequence);
                result = Object::Sequence(sequence);
            }
            i += 1;
        }
        Ok((current_env, result))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Clause {
    For(Vec<Binding>),
    Let(Vec<Binding>),
    Where(Box<dyn Expression>)
}

#[derive(Clone, Debug)]
pub(crate) enum Binding {
    For { name: QName, values: Box<dyn Expression>, st: Option<SequenceType>, allowing_empty: bool, positional_var: Option<QName> },
    Let { name: QName, st: Option<SequenceType>, value: Box<dyn Expression> },
}

#[derive(Clone, Debug)]
pub(crate) struct FLWOR { pub(crate) clauses: Vec<Clause>, pub(crate) return_expr: Box<dyn Expression> }

impl Expression for FLWOR {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        // TODO: new env?
        // TODO: handle  WhereClause | GroupByClause | OrderByClause | CountClause

        let mut pipe = Pipe { binding: None, where_expr: None, return_expr: Some(self.return_expr.clone()), next: None };
        for clause in self.clauses.clone().into_iter().rev() {
            match clause {
                Clause::For(bindings) => {
                    for binding in bindings.into_iter().rev() {
                        pipe = Pipe { binding: Some(binding), where_expr: None, return_expr: None, next: Some(Box::new(pipe)) }
                    }
                },
                Clause::Let(bindings) => {
                    for binding in bindings.into_iter().rev() {
                        pipe = Pipe { binding: Some(binding), where_expr: None, return_expr: None, next: Some(Box::new(pipe)) }
                    }
                },
                Clause::Where(expr) => {
                    pipe = Pipe { where_expr: Some(expr), binding: None, return_expr: None, next: Some(Box::new(pipe)) }
                }
            }
        }

        let (new_env, answer) = eval_pipe(Box::new(pipe), current_env.next(), context)?;
        let current_env = new_env.prev();

        Ok((current_env, answer))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum QuantifiedOp {
    Some,
    Every,
}

#[derive(Clone, Debug)]
pub(crate) struct QuantifiedExpr {
    pub(crate) op: QuantifiedOp,
    pub(crate) name: QName,
    pub(crate) st: Option<SequenceType>,
    pub(crate) seq: Box<dyn Expression>,
    pub(crate) vars: Vec<(QName, Option<SequenceType>, Box<dyn Expression>)>,
    pub(crate) satisfies: Box<dyn Expression>,
}

impl QuantifiedExpr {
    pub(crate) fn boxed(
        op: QuantifiedOp,
        name: QName,
        st: Option<SequenceType>,
        seq: Box<dyn Expression>,
        vars: Vec<(QName, Option<SequenceType>, Box<dyn Expression>)>,
        satisfies: Box<dyn Expression>) -> Box<dyn Expression>
    {
        Box::new(QuantifiedExpr { op, name, st, seq, vars, satisfies })
    }
}

fn process_next(env: Box<Environment>, context: &DynamicContext, op: &QuantifiedOp, index: usize, vars: &Vec<(QName, Option<SequenceType>, Box<dyn Expression>)>, satisfies: &Box<dyn Expression>) -> Result<(Box<Environment>, ControlFlow<bool, bool>), ErrorInfo> {
    if let Some((name, st, seq)) = vars.get(index) {

        let mut current_env = env;

        let (new_env, evaluated) = seq.eval(current_env, context)?;
        current_env = new_env.next();

        let mut result = if op == &QuantifiedOp::Some { false } else { true };

        let name = current_env.namespaces.resolve(&name);
        for mut item in evaluated.into_iter() {
            item = if let Some(st) = &st {
                st.cascade(&current_env, item)?
            } else {
                item
            };
            current_env.set_variable(name.clone(), item);

            let (new_env, state) = process_next(current_env, context, op, index + 1, vars, satisfies)?;
            current_env = new_env;

            match state {
                ControlFlow::Break(_) => {
                    current_env = current_env.prev();
                    return Ok((current_env, state))
                }
                ControlFlow::Continue(v) => result = v,
            }
        }

        current_env = current_env.prev();

        Ok((current_env, ControlFlow::Continue(result)))

    } else {
        let (env, value) = satisfies.eval(env, context)?;

        let state = if value.effective_boolean_value()? {
            if op == &QuantifiedOp::Some {
                ControlFlow::Break(true)
            } else {
                ControlFlow::Continue(true)
            }
        } else {
            if op == &QuantifiedOp::Every {
                ControlFlow::Break(false)
            } else {
                ControlFlow::Continue(false)
            }
        };
        Ok((env, state))
    }
}

impl Expression for QuantifiedExpr {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, evaluated) = self.seq.eval(current_env, context)?;
        current_env = new_env.next();

        let mut result = if self.op == QuantifiedOp::Some { false } else { true };

        let name = current_env.namespaces.resolve(&self.name);
        for mut item in evaluated.into_iter() {
            item = if let Some(st) = &self.st {
                st.cascade(&current_env, item)?
            } else {
                item
            };
            current_env.set_variable(name.clone(), item);

            let (new_env, state) = process_next(current_env, context, &self.op, 0, &self.vars, &self.satisfies)?;
            current_env = new_env;

            match state {
                ControlFlow::Break(v) => {
                    result = v;
                    break;
                }
                ControlFlow::Continue(v) => result = v,
            }
        }

        current_env = current_env.prev();
        Ok((current_env, Object::Atomic(Type::Boolean(result))))
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ArrowExpr {
    source: Box<dyn Expression>,
    name: Option<QName>,
    expr: Option<Box<dyn Expression>>,
    arguments: Vec<Box<dyn Expression>>
}

impl ArrowExpr {
    pub(crate) fn boxed(
        source: Box<dyn Expression>,
        name: Option<QName>,
        expr: Option<Box<dyn Expression>>,
        arguments: Vec<Box<dyn Expression>>
    ) -> Box<dyn Expression> {
        Box::new(ArrowExpr { source, name, expr, arguments })
    }
}

impl Expression for ArrowExpr {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, items) = self.source.eval(current_env, context)?;
        current_env = new_env;

        let mut evaluated_arguments = Vec::with_capacity(self.arguments.len() + 1);
        evaluated_arguments.push(items);

        for argument in &self.arguments {
            let (new_env, value) = argument.eval(current_env, context)?;
            current_env = new_env;

            evaluated_arguments.push(value);
        }

        if let Some(name) = self.name.as_ref() {
            let name = resolve_function_qname(name, &current_env);
            return call(current_env, name, evaluated_arguments, context);
        } else if let Some(expr) = self.expr.as_ref() {
            let (new_env, value) = expr.eval(current_env, context)?;
            current_env = new_env;

            match value {
                Object::FunctionRef { name, arity } => {
                    if arity != evaluated_arguments.len() {
                        todo!("raise error")
                    } else {
                        return call(current_env, name, evaluated_arguments, context);
                    }
                }
                Object::Map(map) => {
                    if evaluated_arguments.len() != 1 {
                        todo!("raise error")
                    } else {
                        match evaluated_arguments.remove(0) {
                            Object::Atomic(t) => {
                                return match map.get(&t) {
                                    Some(v) => Ok((current_env, v.clone())),
                                    None => Ok((current_env, Object::Empty))
                                };
                            }
                            _ => todo!("raise error")
                        }
                    }
                    todo!()
                }
                Object::Array(items) => {
                    if evaluated_arguments.len() != 1 {
                        todo!("raise error")
                    } else {
                        let index = evaluated_arguments.remove(0).to_integer()?;
                        if index >= 1 {
                            return if let Some(item) = items.get((index - 1) as usize) {
                                Ok((current_env, item.clone()))
                            } else {
                                Ok((current_env, Object::Empty))
                            };
                        } else {
                            todo!("raise error?")
                        }
                    }
                    todo!()
                }
                _ => return Err((ErrorCode::XPTY0004, format!("{:?}", value)))
            }
        } else {
            panic!("internal error")
        }
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchExpr {
    source: Box<dyn Expression>,
    clauses: Vec<SwitchCaseClause>,
    default_expr: Box<dyn Expression>
}

impl SwitchExpr {
    pub(crate) fn boxed(
        source: Box<dyn Expression>,
        clauses: Vec<SwitchCaseClause>,
        default_expr: Box<dyn Expression>
    ) -> Box<dyn Expression> {
        Box::new(SwitchExpr { source, clauses, default_expr })
    }
}

impl Expression for SwitchExpr {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, mut source) = self.source.eval(current_env, context)?;
        current_env = new_env;

        source = atomization(&current_env, source)?;

        for clause in &self.clauses {

            for operand in &clause.operands {
                let (new_env, mut value) = operand.eval(current_env, context)?;
                current_env = new_env;

                value = atomization(&current_env, value)?;

                match comparison::deep_eq((&current_env, &source), (&current_env, &value)) {
                    Ok(v) => {
                        if v { return clause.expr.eval(current_env, context); }
                    },
                    _ => {}
                }
            }
        }

        self.default_expr.eval(current_env, context)
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchCaseClause {
    operands: Vec<Box<dyn Expression>>,
    expr: Box<dyn Expression>
}

impl SwitchCaseClause {
    pub(crate) fn new(
        operands: Vec<Box<dyn Expression>>,
        expr: Box<dyn Expression>
    ) -> Self {
        SwitchCaseClause { operands, expr }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TypeswitchExpr {
    source: Box<dyn Expression>,
    clauses: Vec<CaseClause>,
    default_name: Option<QName>,
    default_expr: Box<dyn Expression>
}

impl TypeswitchExpr {
    pub(crate) fn boxed(
        source: Box<dyn Expression>,
        clauses: Vec<CaseClause>,
        default_name: Option<QName>,
        default_expr: Box<dyn Expression>
    ) -> Box<dyn Expression> {
        Box::new(TypeswitchExpr { source, clauses, default_name, default_expr })
    }
}

impl Expression for TypeswitchExpr {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let mut current_env = env;

        let (new_env, items) = self.source.eval(current_env, context)?;
        current_env = new_env;

        for clause in &self.clauses {
            for st in &clause.stu {
                match st.check(&current_env, &items) {
                    Ok(flag) => {
                        if flag {

                            let mut env = current_env.next();

                            if let Some(name) = clause.name.as_ref() {
                                let name = resolve_element_qname(&name, &env);
                                env.set_variable(name, items)
                            }

                            let (new_env, result) = clause.expr.eval(env, context)?;
                            current_env = new_env.prev();

                            return Ok((current_env, result))
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        let mut env = current_env.next();

        if let Some(name) = self.default_name.as_ref() {
            let name = resolve_element_qname(&name, &env);
            env.set_variable(name, items)
        }

        let (new_env, result) = self.default_expr.eval(env, context)?;
        current_env = new_env.prev();

        return Ok((current_env, result))
    }

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CaseClause {
    name: Option<QName>,
    stu: Vec<SequenceType>,
    expr: Box<dyn Expression>
}

impl CaseClause {
    pub(crate) fn new(
        name: Option<QName>,
        stu: Vec<SequenceType>,
        expr: Box<dyn Expression>
    ) -> Self {
        CaseClause { name, stu, expr }
    }
}