use super::{
    Expr, array, call, function, identifier, literal, object, operator, quantified, reason,
    template, version,
};
use crate::error::{Error, Result};
use crate::runtime::{MAX_DEPTH, Runtime};
use crate::scope::Scope;
use crate::value::Value;

/// An expression, dispatched to the kind that knows how to evaluate it.
///
/// Instantiation uses call syntax rather than a separate variant;
/// [`new::New`](super::new::New) recognizes calls to class names.
#[derive(Debug)]
pub enum Ast<'a> {
    Literal(literal::Literal<'a>),
    Identifier(identifier::Identifier<'a>),
    Array(array::Array<'a>),
    Object(object::Object<'a>),
    Function(function::Function<'a>),
    Call(call::Call<'a>),
    Template(template::Template<'a>),
    Operator(operator::Operator<'a>),
    Reason(reason::Reason<'a>),
    Version(version::Version<'a>),
    Quantified(quantified::Quantified<'a>),
}

impl<'a> Ast<'a> {
    #[must_use]
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
            Expr::ObjectLiteral(_) | Expr::SpreadObject(_) | Expr::Enum(_) => {
                Ast::Object(object::Object::new(node))
            }
            Expr::Function(_) => Ast::Function(function::Function::new(node)),
            Expr::Call { .. } | Expr::Super(_) | Expr::Of { .. } => {
                Ast::Call(call::Call::new(node))
            }
            Expr::Template(_) => Ast::Template(template::Template::new(node)),
            Expr::Unary { .. }
            | Expr::Binary { .. }
            | Expr::Logical { .. }
            | Expr::Conditional { .. }
            | Expr::Assign { .. }
            | Expr::Delete(_) => Ast::Operator(operator::Operator::new(node)),
            Expr::Reason { .. } | Expr::Depth { .. } | Expr::Path { .. } | Expr::Model => {
                Ast::Reason(reason::Reason::new(node))
            }
            Expr::Version { .. } => Ast::Version(version::Version::new(node)),
            Expr::Quantified { .. } => Ast::Quantified(quantified::Quantified::new(node)),
        }
    }

    /// Evaluates this node in the given scope.
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
            Ast::Reason(node) => node.resolve(runtime, scope),
            Ast::Version(node) => node.resolve(runtime, scope),
            Ast::Quantified(node) => node.resolve(runtime, scope),
        }
    }

    #[must_use]
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
            Ast::Reason(node) => node.node,
            Ast::Version(node) => node.node,
            Ast::Quantified(node) => node.node,
        }
    }

    /// Renders this node as source.
    #[must_use]
    pub fn generate(&self) -> String {
        self.node().to_string()
    }

    #[must_use]
    pub fn first(&self) -> Option<&'a Expr> {
        self.node().first()
    }

    #[must_use]
    pub fn object(&self) -> Option<&'a Expr> {
        self.node().object()
    }

    #[must_use]
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

        let depth = self.depth;
        let result = Ast::convert(expression).resolve(self, scope);
        debug_assert_eq!(self.depth, depth);
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
}
