//! Questions asked of a whole expression tree, rather than of one node.
//!
//! Each query visits the children relevant to its question.
//! [`Expression::roots`] stops at a function body because a lambda's parameters
//! are not names the surrounding declaration reads.

use crate::error::Result;
use crate::lang::ast::{Expr, FunctionBody, TemplatePart};
use crate::runtime::Runtime;
use crate::scope::Scope;

pub struct Expression<'a> {
    pub node: &'a Expr,
}

impl<'a> Expression<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Expression { node }
    }

    /// The bare names this expression reads.
    ///
    /// A function body is not descended into: its parameters are bound by the
    /// function, so a rule that passes a lambda does not read them.
    /// Without runtime routing, this syntactic query assumes ordinary calls;
    /// runtime rule checks use context-aware collection for constructor labels.
    pub fn roots(&self) -> Vec<String> {
        self.roots_with(false, &|_| false)
    }

    /// Resolve constructor routing without reclassifying function assignments.
    pub(crate) fn roots_with(
        &self,
        constructor: bool,
        resolve: &impl Fn(&Expr) -> bool,
    ) -> Vec<String> {
        let mut found = Vec::new();
        roots(self.node, constructor, resolve, false, &mut found);
        found
    }

    /// Static reference validation must not reject a deliberately deferred
    /// bare-slot version. Its receivers and computed keys are still strict.
    pub(crate) fn required_roots_with(
        &self,
        constructor: bool,
        resolve: &impl Fn(&Expr) -> bool,
    ) -> Vec<String> {
        let mut found = Vec::new();
        roots(self.node, constructor, resolve, true, &mut found);
        found
    }

    /// The first `$Class` this expression mentions, if any.
    pub fn class_reference(&self) -> Option<String> {
        let mut found = None;
        class_reference(self.node, &mut found);
        found
    }

    /// The `$Class.property` names this expression reads, for the given class.
    pub fn class_properties(&self, class: &str) -> Vec<String> {
        let mut found = Vec::new();
        class_properties(self.node, class, None, &mut found);
        found
    }

    /// Only a fixed history of the rule's own destination is a self-anchor.
    /// Other property reads keep the accepted class-cycle checks.
    pub(crate) fn class_rule_properties(&self, class: &str, destination: &str) -> Vec<String> {
        let mut found = Vec::new();
        class_properties(self.node, class, Some(destination), &mut found);
        found
    }

    /// Validate calls before a static reference check inspects their values.
    /// Like roots, this does not enter deferred function bodies.
    pub(crate) fn check_calls(&self, check: &impl Fn(&Expr) -> Result<()>) -> Result<()> {
        check_calls(self.node, check)
    }
}

fn check_calls(expression: &Expr, check: &impl Fn(&Expr) -> Result<()>) -> Result<()> {
    if matches!(expression, Expr::Call { .. } | Expr::Super(_)) {
        check(expression)?;
    }
    let mut result = Ok(());
    visit_children(expression, &mut |child| {
        if result.is_ok() {
            result = check_calls(child, check);
        }
    });
    result
}

impl Runtime {
    pub(crate) fn expression_roots(
        &self,
        expression: &Expr,
        scope: &Scope,
        assignment_instantiation: bool,
    ) -> Vec<String> {
        let resolve = |expression: &Expr| self.is_constructor_call(expression, scope);
        let constructor =
            (assignment_instantiation && self.is_instantiation(expression)) || resolve(expression);
        Expression::new(expression).roots_with(constructor, &resolve)
    }

    pub(crate) fn required_expression_roots(
        &self,
        expression: &Expr,
        scope: &Scope,
    ) -> Vec<String> {
        let resolve = |expression: &Expr| self.is_constructor_call(expression, scope);
        let constructor = self.is_instantiation(expression) || resolve(expression);
        Expression::new(expression).required_roots_with(constructor, &resolve)
    }
}

fn roots(
    expression: &Expr,
    constructor: bool,
    resolve: &impl Fn(&Expr) -> bool,
    required: bool,
    found: &mut Vec<String>,
) {
    match expression {
        Expr::Version { reference, .. }
            if required && matches!(reference.as_ref(), Expr::Identifier(_)) => {}
        Expr::Identifier(name) => found.push(name.clone()),
        Expr::Call { arguments, .. } | Expr::Super(arguments) => {
            if let Expr::Call { callee, .. } = expression {
                roots(callee, resolve(callee), resolve, required, found);
            }
            for argument in arguments {
                let value = if constructor {
                    argument
                        .named_argument()
                        .map_or(argument, |(_, value)| value)
                } else {
                    argument
                };
                roots(value, resolve(value), resolve, required, found);
            }
        }
        _ => visit_children(expression, &mut |child| {
            roots(child, resolve(child), resolve, required, found)
        }),
    }
}

fn class_reference(expression: &Expr, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match expression {
        Expr::ClassRef(name) => *found = Some(name.clone()),
        // A rule written as a lambda still states something about the type, so
        // unlike `roots` this does look inside one.
        Expr::Function(function) => {
            if let FunctionBody::Expression(body) = &function.body {
                class_reference(body, found);
            }
        }
        _ => visit_children(expression, &mut |child| class_reference(child, found)),
    }
}

fn class_properties(
    expression: &Expr,
    class: &str,
    destination: Option<&str>,
    found: &mut Vec<String>,
) {
    match expression {
        Expr::Version {
            reference,
            selector,
        } if selector.is_fixed()
            && destination.is_some_and(|property| class_property(reference, class, property)) => {}
        Expr::Member { property, .. } if property == "value" => {}
        Expr::Member { object, property } => {
            if matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
                found.push(property.clone());
            } else {
                class_properties(object, class, destination, found);
            }
        }
        Expr::Index { object, index } => {
            if let (Expr::ClassRef(name), Expr::String(property)) =
                (object.as_ref(), index.as_ref())
            {
                if name == class {
                    found.push(property.clone());
                }
            }
            visit_children(expression, &mut |child| {
                class_properties(child, class, destination, found)
            });
        }
        _ => visit_children(expression, &mut |child| {
            class_properties(child, class, destination, found)
        }),
    }
}

fn class_property(expression: &Expr, class: &str, property: &str) -> bool {
    match expression {
        Expr::Member {
            object,
            property: name,
        } => matches!(object.as_ref(), Expr::ClassRef(name) if name == class) && name == property,
        Expr::Index { object, index } => {
            matches!(object.as_ref(), Expr::ClassRef(name) if name == class)
                && matches!(index.as_ref(), Expr::String(name) if name == property)
        }
        _ => false,
    }
}

/// Visits immediate expression children without entering a function's scope.
fn visit_children(expression: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expression {
        Expr::Member { object, .. } => visit(object),
        Expr::Version { reference, .. } => visit(reference),
        Expr::Index { object, index } => {
            visit(object);
            visit(index);
        }
        Expr::Slice { object, start, end } => {
            visit(object);
            if let Some(start) = start {
                visit(start);
            }
            if let Some(end) = end {
                visit(end);
            }
        }
        Expr::Call { callee, arguments } => {
            visit(callee);
            for argument in arguments {
                visit(argument);
            }
        }
        Expr::Unary { operand, .. } | Expr::Delete(operand) => visit(operand),
        Expr::Reason { source, .. } => visit(source),
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Assign {
            target: left,
            value: right,
        } => {
            visit(left);
            visit(right);
        }
        Expr::List(items) | Expr::Super(items) => {
            for item in items {
                visit(item);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                visit(value);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    visit(expression);
                }
            }
        }
        Expr::Null
        | Expr::Bool(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::Regex(_)
        | Expr::Identifier(_)
        | Expr::ClassRef(_)
        | Expr::ObjectRef(_)
        | Expr::This
        | Expr::Function(_)
        | Expr::Model => {}
    }
}
