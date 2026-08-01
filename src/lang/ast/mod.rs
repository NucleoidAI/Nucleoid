//! The syntax tree, and how each kind of node is evaluated.
//!
//! [`Expr`] and [`Stmt`] are the tree the parser builds. [`Ast`] is
//! `ref/src/lang/ast/Node.js`: the base class every node kind shares, as a
//! closed enum, with [`Ast::convert`] standing in for `Node.convert` and one
//! type per `ref/src/lang/ast/*.js` behind it.
//!
//! `ref` resolves a node into a rewritten ESTree and hands it to JavaScript's
//! `eval`, so its `resolve` returns a tree and `generate` turns that tree into
//! the string that gets evaluated. There is no `eval` here: [`Ast::resolve`]
//! produces the value directly, and [`Ast::generate`] is only used to key a
//! statement in the graph. `Node.graph(scope)`, which walks a node for the
//! identifiers it reads, has no counterpart — reads are recorded as they
//! happen, in [`crate::lang::evaluation`], rather than predicted statically.

pub mod array;
pub mod call;
pub mod function;
pub mod identifier;
pub mod literal;
pub mod new;
pub mod object;
pub mod operator;
pub mod template;

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::runtime::{MAX_DEPTH, Runtime};
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
        target: Expr,
        value: Expr,
    },
    Expression(Expr),
    If {
        condition: Expr,
        consequent: Vec<Stmt>,
        alternate: Option<Box<Stmt>>,
    },
    Block(Vec<Stmt>),
    Class(ClassDecl),
    Function(Arc<Function>),
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Throw(Expr),
    Delete(Expr),
    Try {
        body: Vec<Stmt>,
        parameter: String,
        catch: Vec<Stmt>,
    },
    /// A declaration with a type but no definition, such as `a: int`.
    Declaration {
        name: String,
        type_name: String,
    },
    Pass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<String>,
    pub parameters: Vec<Parameter>,
    pub fields: Vec<Parameter>,
    pub constructor: Vec<Stmt>,
    pub methods: Vec<Arc<Function>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub parameters: Vec<Parameter>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Expression(Box<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Template(Vec<TemplatePart>),
    Regex(String),
    Identifier(String),
    /// A class reference written as `$Person`.
    ClassRef(String),
    /// A resolved object, used when a statement is rebuilt for the graph so it
    /// re-runs against the same object rather than re-resolving a path.
    ObjectRef(String),
    This,
    Super(Vec<Expr>),
    Member {
        object: Box<Expr>,
        property: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Logical {
        operator: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    List(Vec<Expr>),
    ObjectLiteral(Vec<(String, Expr)>),
    Function(Arc<Function>),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Delete(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
    Plus,
    TypeOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

impl Expr {
    /// The dotted path this expression denotes, when it is a plain identifier
    /// chain such as `a`, `person1.name` or `$Person.mortal`.
    pub fn path(&self) -> Option<String> {
        match self {
            Expr::Identifier(name) => Some(name.clone()),
            Expr::ClassRef(name) => Some(name.clone()),
            Expr::ObjectRef(id) => Some(id.clone()),
            Expr::Member { object, property } => {
                object.path().map(|base| format!("{base}.{property}"))
            }
            _ => None,
        }
    }

    /// The leftmost node of a path expression. `Node.first` in
    /// `ref/src/lang/ast/Identifier.js`.
    pub fn first(&self) -> Option<&Expr> {
        match self {
            Expr::Identifier(_) | Expr::ClassRef(_) | Expr::ObjectRef(_) | Expr::This => Some(self),
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => object.first(),
            Expr::Call { callee, .. } => callee.first(),
            _ => None,
        }
    }

    /// What this node is read from — the `a.b` of `a.b.c`. `Node.object`.
    pub fn object(&self) -> Option<&Expr> {
        match self {
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => Some(object),
            _ => None,
        }
    }

    /// The rightmost name of a path expression — the `c` of `a.b.c`.
    /// `Node.last`.
    pub fn last(&self) -> Option<&str> {
        match self {
            Expr::Identifier(name) | Expr::ClassRef(name) | Expr::ObjectRef(name) => Some(name),
            Expr::Member { property, .. } => Some(property),
            _ => None,
        }
    }
}

/// An expression, dispatched to the kind that knows how to evaluate it.
///
/// This is `Node.convert` in `ref/src/lang/ast/Node.js`: the same nine kinds,
/// as a closed enum rather than a class hierarchy. `ref`'s `New` is missing a
/// variant because Nucleoid has no `new` keyword — an instantiation is an
/// ordinary call until the name turns out to be a class, which is what
/// [`new::New`] decides.
pub enum Ast<'a> {
    Literal(literal::Literal<'a>),
    Identifier(identifier::Identifier<'a>),
    Array(array::Array<'a>),
    Object(object::Object<'a>),
    Function(function::Function<'a>),
    Call(call::Call<'a>),
    Template(template::Template<'a>),
    Operator(operator::Operator<'a>),
}

impl<'a> Ast<'a> {
    pub fn convert(node: &'a Expr) -> Ast<'a> {
        match node {
            Expr::Null | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Regex(_) => {
                Ast::Literal(literal::Literal::new(node))
            }
            Expr::Identifier(_)
            | Expr::ClassRef(_)
            | Expr::ObjectRef(_)
            | Expr::This
            | Expr::Member { .. } => Ast::Identifier(identifier::Identifier::new(node)),
            Expr::List(_) | Expr::Index { .. } | Expr::Slice { .. } => {
                Ast::Array(array::Array::new(node))
            }
            Expr::ObjectLiteral(_) => Ast::Object(object::Object::new(node)),
            Expr::Function(_) => Ast::Function(function::Function::new(node)),
            Expr::Call { .. } | Expr::Super(_) => Ast::Call(call::Call::new(node)),
            Expr::Template(_) => Ast::Template(template::Template::new(node)),
            Expr::Unary { .. }
            | Expr::Binary { .. }
            | Expr::Logical { .. }
            | Expr::Assign { .. }
            | Expr::Delete(_) => Ast::Operator(operator::Operator::new(node)),
        }
    }

    /// `Node.resolve(scope)` — the value this node has in this scope.
    ///
    /// `ref` resolves a node into a rewritten tree and hands it to JavaScript's
    /// `eval`; there is no such step here, so resolving *is* evaluating.
    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self {
            Ast::Literal(node) => node.resolve(),
            Ast::Identifier(node) => node.resolve(runtime, scope),
            Ast::Array(node) => node.resolve(runtime, scope),
            Ast::Object(node) => node.resolve(runtime, scope),
            Ast::Function(node) => node.resolve(),
            Ast::Call(node) => node.resolve(runtime, scope),
            Ast::Template(node) => node.resolve(runtime, scope),
            Ast::Operator(node) => node.resolve(runtime, scope),
        }
    }

    pub fn node(&self) -> &'a Expr {
        match self {
            Ast::Literal(node) => node.node,
            Ast::Identifier(node) => node.node,
            Ast::Array(node) => node.node,
            Ast::Object(node) => node.node,
            Ast::Function(node) => node.node,
            Ast::Call(node) => node.node,
            Ast::Template(node) => node.node,
            Ast::Operator(node) => node.node,
        }
    }

    /// `Node.generate(scope)` — the node written back as source.
    pub fn generate(&self) -> String {
        self.node().to_string()
    }

    pub fn first(&self) -> Option<&'a Expr> {
        self.node().first()
    }

    pub fn object(&self) -> Option<&'a Expr> {
        self.node().object()
    }

    pub fn last(&self) -> Option<&'a str> {
        self.node().last()
    }
}

impl Runtime {
    pub(crate) fn evaluate(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Value> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }

        let result = Ast::convert(expression).resolve(self, scope);
        self.depth -= 1;
        result
    }

    pub(crate) fn evaluate_all(
        &mut self,
        expressions: &[Expr],
        scope: &mut Scope,
    ) -> Result<Vec<Value>> {
        let mut values = Vec::with_capacity(expressions.len());

        for expression in expressions {
            values.push(self.evaluate(expression, scope)?);
        }

        Ok(values)
    }

    /// Renders a path for an error message, preferring the source spelling.
    pub(crate) fn describe(&self, expression: &Expr, _scope: &Scope) -> String {
        expression.path().unwrap_or_else(|| expression.to_string())
    }
}

// -- walking -----------------------------------------------------------------
//
// `ref/src/lang/ast/Node.js` exposes these as `Node.walk()`.

/// The bare names a statement assigns to.
pub(crate) fn collect_assigned_names(statement: &Stmt, names: &mut Vec<String>) {
    match statement {
        Stmt::Assign {
            target: Expr::Identifier(name),
            ..
        } => names.push(name.clone()),
        Stmt::Block(nested) => {
            for statement in nested {
                collect_assigned_names(statement, names);
            }
        }
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            for statement in consequent {
                collect_assigned_names(statement, names);
            }
            if let Some(alternate) = alternate {
                collect_assigned_names(alternate, names);
            }
        }
        _ => {}
    }
}

/// The leftmost names a statement reads.
pub(crate) fn collect_read_roots(statement: &Stmt, roots: &mut Vec<String>) {
    match statement {
        Stmt::Assign { target, value } => {
            if let Expr::Member { object, .. } = target {
                collect_roots(object, roots);
            }
            collect_roots(value, roots);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            collect_roots(expression, roots)
        }
        Stmt::Return(Some(expression)) => collect_roots(expression, roots),
        Stmt::Block(nested) => {
            for statement in nested {
                collect_read_roots(statement, roots);
            }
        }
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            collect_roots(condition, roots);
            for statement in consequent {
                collect_read_roots(statement, roots);
            }
            if let Some(alternate) = alternate {
                collect_read_roots(alternate, roots);
            }
        }
        _ => {}
    }
}

pub(crate) fn collect_roots(expression: &Expr, roots: &mut Vec<String>) {
    match expression {
        Expr::Identifier(name) => roots.push(name.clone()),
        Expr::Member { object, .. } | Expr::Slice { object, .. } => collect_roots(object, roots),
        Expr::Index { object, index } => {
            collect_roots(object, roots);
            collect_roots(index, roots);
        }
        Expr::Call { callee, arguments } => {
            collect_roots(callee, roots);
            for argument in arguments {
                collect_roots(argument, roots);
            }
        }
        Expr::Unary { operand, .. } => collect_roots(operand, roots),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            collect_roots(left, roots);
            collect_roots(right, roots);
        }
        Expr::List(items) => {
            for item in items {
                collect_roots(item, roots);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                collect_roots(value, roots);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    collect_roots(expression, roots);
                }
            }
        }
        Expr::Assign { target, value } => {
            collect_roots(target, roots);
            collect_roots(value, roots);
        }
        _ => {}
    }
}

/// The `$Class.property` names an expression reads.
pub(crate) fn collect_class_properties(expression: &Expr, class: &str, found: &mut Vec<String>) {
    match expression {
        Expr::Member { object, property } => {
            if matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
                found.push(property.clone());
            } else {
                collect_class_properties(object, class, found);
            }
        }
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            collect_class_properties(left, class, found);
            collect_class_properties(right, class, found);
        }
        Expr::Unary { operand, .. } => collect_class_properties(operand, class, found),
        Expr::Call { callee, arguments } => {
            collect_class_properties(callee, class, found);
            for argument in arguments {
                collect_class_properties(argument, class, found);
            }
        }
        Expr::Index { object, index } => {
            collect_class_properties(object, class, found);
            collect_class_properties(index, class, found);
        }
        _ => {}
    }
}

pub(crate) fn find_class_reference_statement(statement: &Stmt, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match statement {
        Stmt::Assign { target, value } => {
            find_class_reference(target, found);
            find_class_reference(value, found);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            find_class_reference(expression, found)
        }
        Stmt::Return(Some(expression)) => find_class_reference(expression, found),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            find_class_reference(condition, found);
            for statement in consequent {
                find_class_reference_statement(statement, found);
            }
            if let Some(alternate) = alternate {
                find_class_reference_statement(alternate, found);
            }
        }
        Stmt::Block(statements) => {
            for statement in statements {
                find_class_reference_statement(statement, found);
            }
        }
        _ => {}
    }
}

pub(crate) fn find_class_reference(expression: &Expr, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match expression {
        Expr::ClassRef(name) => *found = Some(name.clone()),
        Expr::Member { object, .. } => find_class_reference(object, found),
        Expr::Index { object, index } => {
            find_class_reference(object, found);
            find_class_reference(index, found);
        }
        Expr::Slice { object, start, end } => {
            find_class_reference(object, found);
            if let Some(start) = start {
                find_class_reference(start, found);
            }
            if let Some(end) = end {
                find_class_reference(end, found);
            }
        }
        Expr::Call { callee, arguments } => {
            find_class_reference(callee, found);
            for argument in arguments {
                find_class_reference(argument, found);
            }
        }
        Expr::Unary { operand, .. } => find_class_reference(operand, found),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            find_class_reference(left, found);
            find_class_reference(right, found);
        }
        Expr::List(items) => {
            for item in items {
                find_class_reference(item, found);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                find_class_reference(value, found);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    find_class_reference(expression, found);
                }
            }
        }
        Expr::Assign { target, value } => {
            find_class_reference(target, found);
            find_class_reference(value, found);
        }
        Expr::Function(function) => {
            if let FunctionBody::Expression(body) = &function.body {
                find_class_reference(body, found);
            }
        }
        _ => {}
    }
}
