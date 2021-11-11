use crate::eval::expression::{Expression, NodeTest};
use crate::parser::op::{Representation, OperatorArithmetic, OperatorComparison};
use bigdecimal::BigDecimal;
use ordered_float::OrderedFloat;
use crate::values::{QName, resolve_function_qname, resolve_element_qname};
use crate::fns::{Param, call, object_to_bool};
use crate::eval::{Environment, DynamicContext, EvalResult, Object, Type, eval_predicates, Axis, step_and_test, object_to_qname, object_owned_to_sequence, object_to_integer, ErrorInfo, INS};
use crate::serialization::{object_to_string};
use crate::serialization::to_string::object_to_string_xml;
use crate::eval::helpers::{relax, relax_sequences, sort_and_dedup, process_items, join_sequences};
use std::collections::HashMap;
use crate::eval::arithmetic::{eval_unary, eval_arithmetic};
use crate::eval::comparison::{eval_comparison, eval_comparison_item};
use crate::eval::piping::{Pipe, eval_pipe};
use crate::parser::errors::{CustomError, ErrorCode};
use crate::eval::sequence_type::SequenceType;
use linked_hash_map::LinkedHashMap;
use crate::namespaces::Namespace;

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
    pub(crate) fn boxed<I>(encoding: Option<String>, version: Option<String>) -> Result<Box<dyn Expression>, (CustomError<I>, String)> {
        if let Some(version) = &version {
            match version.as_str() {
                "1.0" | "3.0" | "3.1" => {
                    // TODO
                }
                _ => return Err((CustomError::XQST0031, format!("unsupported version {}", version)))
            }
        }
        if let Some(encoding) = &encoding {
            match encoding.to_uppercase().replace("&#X2D;", "-").as_str() {
                "US-ASCII" |
                "ISO-8859-1" |
                "UTF-8" => {
                    // TODO
                },
                _ => return Err((CustomError::XQST0087, format!("unsupported encoding {}", encoding)))
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

#[derive(Clone, Debug)]
pub(crate) struct DeclareDecimalFormat {
    format: Option<QName>,
    properties: Vec<(String, String)>,
}

impl DeclareDecimalFormat {
    pub(crate) fn boxed(format: Option<QName>, properties: Vec<(String, String)>) -> Box<dyn Expression> {
        Box::new(DeclareDecimalFormat { format, properties })
    }
}

impl Expression for DeclareDecimalFormat {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
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

        let ns = Namespace { prefix, uri };

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
                new_env.namespaces.default_for_element = uri;
            }
            "function" => {
                new_env.namespaces.default_for_function = uri;
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
                    env.set(name, obj);
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
    pub(crate) type_declaration: Option<SequenceType>,
    pub(crate) external: bool,
    pub(crate) body: Option<Box<dyn Expression>>
}

impl Expression for FunctionDecl {
    fn eval<'a>(&self, mut env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let name = resolve_function_qname(&self.name, &env);

        // TODO: handle typeDeclaration

        if let Some(body) = self.body.clone() {
            env.functions.put(name, self.params.clone(), body);

        } else {
            panic!("error")
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
    pub(crate) predicates: Vec<Predicate>
}

impl AxisStep {
    pub(crate) fn boxed(step: Box<dyn Expression>, predicates: Vec<Predicate>) -> Box<dyn Expression> {
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
        step_and_test(&self.axis, &self.test, env, context)
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
    pub(crate) fn new(exprs: Vec<Box<dyn Expression>>) -> Box<Self> {
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

            let str = object_to_string(&current_env, &object);
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
pub(crate) struct Predicate { pub(crate) expr: Box<dyn Expression> }

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

        process_items(current_env, object, |env, item| {
            let result = self.st.is_castable(&item)?;
            Ok((env, Object::Atomic(Type::Boolean(result))))
        })
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

        process_items(current_env, object, |env, item| {
            let correct = self.st.is_castable(&item)?;

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

        println!("st {:?}", self.st);

        Ok((new_env, object))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Cast { pub(crate) expr: Box<dyn Expression>, pub(crate) st: SequenceType }

impl Expression for Cast {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        todo!()
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Postfix { pub(crate) primary: Box<dyn Expression>, pub(crate) suffix: Vec<Predicate> }

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

            let mut items = object_owned_to_sequence(items);

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
            let (new_env, evaluated_child) = child.eval(current_env, context)?;
            current_env = new_env;

            match evaluated_child {
                Object::Empty => {},
                Object::Sequence(items) => {
                    let mut add_space = false;
                    for item in items {
                        match item {
                            Object::Node(rf) => {
                                if current_env.xml_tree_id() != rf.xml_tree_id() {
                                    current_env.xml_writer(|w| w.link_node(&rf));
                                }
                            },
                            Object::Atomic(..) => {
                                let mut content = object_to_string_xml(&current_env, &item);
                                if add_space {
                                    content.insert(0, ' ');
                                }

                                current_env.xml_writer(|w| w.text(content));

                                add_space = true;
                            }
                            _ => panic!("unexpected object {:?}", item) //TODO: better error
                        }
                    }
                },
                Object::Node(rf) => {
                    if current_env.xml_tree_id() != rf.xml_tree_id() {
                        current_env.xml_writer(|w| w.link_node(&rf));
                    }
                },
                Object::Atomic(..) => {
                    let content = object_to_string(&current_env, &evaluated_child);
                    current_env.xml_writer(|w| w.text(content));
                }
                _ => panic!("unexpected object {:?}", evaluated_child) //TODO: better error
            };
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

        process_items(current_env, evaluated, |env, item| {
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

        let it = object_owned_to_sequence(value);

        let mut evaluated = vec![];

        let last = Some(it.len());
        let mut position = 0;
        for item in it {
            position += 1;
            let context = DynamicContext {
                initial_node_sequence: None,
                item, position: Some(position), last
            };

            let (new_env, l_value) = self.left.eval(current_env, &context)?;
            current_env = new_env;

            let (new_env, r_value) = self.right.eval(current_env, &context)?;
            current_env = new_env;

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

        process_items(current_env, evaluated, |env, item| {
            let v = match object_to_bool(&item) {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            if v {
                let (new_env, evaluated) = self.consequence.eval(env, context)?;

                Ok((new_env, evaluated))
            } else {
                let (new_env, evaluated) = self.alternative.eval(env, context)?;

                Ok((new_env, evaluated))
            }
        })
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Function {
    pub(crate) arguments: Vec<Param>,
    pub(crate) body: Box<dyn Expression>
}

impl Expression for Function {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Function { parameters: self.arguments.clone(), body: self.body.clone() }))
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

        let (parameters, body) = match current_env.get(&name) {
            Some(Object::Function {parameters, body}) => (parameters, body),
            None => {
                let mut evaluated_arguments = vec![];
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
            fn_env.set(name, value);
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

            let v = object_to_bool(&result)?;
            if v {
                evaluated.push(context.item)
            }
        }

        relax(current_env, evaluated)
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
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        let name = resolve_element_qname(&self.name, &env);

        if let Some(value) = env.get(&name) {
            Ok((env, value))
        } else {
            panic!("unknown variable {:?}", name)
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
                let mut acc = true;
                for item in sequence {
                    match object_to_bool(&item) {
                        Ok(v) => acc = acc || v,
                        Err(e) => return Err(e)
                    }
                }

                Ok((current_env, Object::Atomic(Type::Boolean(acc))))
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
                let mut acc = true;
                for item in sequence {
                    match object_to_bool(&item) {
                        Ok(v) => {
                            if !v {
                                acc = false;
                                break;
                            }
                        },
                        Err(e) => return Err(e)
                    }
                }

                Object::Atomic(Type::Boolean(acc))
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
