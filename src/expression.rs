//! Questions asked of a whole expression tree, rather than of one node.
//!
//! Each query visits the children relevant to its question.
//! [`Expression::roots`] stops at a function body because a lambda's parameters
//! are not names the surrounding declaration reads.

use crate::error::Result;
use crate::lang::ast::{BinaryOp, Expr, FunctionBody, TemplatePart};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug)]
pub struct Expression<'a> {
    pub node: &'a Expr,
}

/// The `$Class` properties one class rule reads.
#[derive(Debug, Default)]
pub(crate) struct ClassReads {
    /// Read live, at a moving history position, or through any other path.
    pub(crate) live: Vec<String>,
    /// Read only at `@first` or `@n`, written directly on `$Class`. A fixed
    /// position never changes during its slot's lifetime.
    pub(crate) fixed: Vec<String>,
}

impl<'a> Expression<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Expression { node }
    }

    /// The bare names this expression reads.
    ///
    /// A function body is not descended into: its parameters are bound by the
    /// function, so a rule that passes a lambda does not read them.
    /// Without runtime routing, this syntactic query assumes ordinary calls;
    /// runtime rule checks use context-aware collection for constructor labels.
    #[must_use]
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
    #[must_use]
    pub fn class_reference(&self) -> Option<String> {
        let mut found = None;
        class_reference(self.node, &mut found);
        found
    }

    /// The `$Class.property` names this expression reads, for the given class.
    #[must_use]
    pub fn class_properties(&self, class: &str) -> Vec<String> {
        let mut reads = ClassReads::default();
        class_properties(self.node, class, false, &mut reads);
        reads.live
    }

    /// The class properties a rule reads, with those it names only at a fixed
    /// history position directly on `$Class` kept apart as anchors.
    pub(crate) fn class_rule_reads(&self, class: &str) -> ClassReads {
        let mut reads = ClassReads::default();
        class_properties(self.node, class, true, &mut reads);
        reads
            .fixed
            .retain(|property| !reads.live.contains(property));
        reads
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
        let resolve = |call: &Expr| self.is_constructor_call(call, scope);
        let constructor =
            (assignment_instantiation && self.is_instantiation(expression)) || resolve(expression);
        Expression::new(expression).roots_with(constructor, &resolve)
    }

    pub(crate) fn required_expression_roots(
        &self,
        expression: &Expr,
        scope: &Scope,
    ) -> Vec<String> {
        let resolve = |call: &Expr| self.is_constructor_call(call, scope);
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
        Expr::Binary {
            operator: BinaryOp::Is | BinaryOp::IsNot,
            left,
            right,
        } if required && matches!(right.as_ref(), Expr::Identifier(_)) => {
            roots(left, resolve(left), resolve, required, found);
        }
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
        Expr::Quantified {
            binder,
            collection,
            condition,
            ..
        } => {
            roots(collection, resolve(collection), resolve, required, found);
            let mut bound = Vec::new();
            roots(condition, resolve(condition), resolve, required, &mut bound);
            found.extend(bound.into_iter().filter(|name| name != binder));
        }
        _ => visit_children(expression, &mut |child| {
            roots(child, resolve(child), resolve, required, found);
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

/// Collects class property reads. With `anchors`, a fixed history read written
/// directly on `$Class` is reported as fixed rather than as a live read.
fn class_properties(expression: &Expr, class: &str, anchors: bool, reads: &mut ClassReads) {
    match expression {
        Expr::Version {
            reference,
            selector,
        } if anchors && selector.is_fixed() => match class_property(reference, class) {
            Some(property) => reads.fixed.push(property.to_string()),
            None => visit_children(expression, &mut |child| {
                class_properties(child, class, anchors, reads);
            }),
        },
        Expr::Member { property, .. } if property == "value" => {}
        Expr::Member { object, property } => {
            member_properties(object, property, class, anchors, reads);
        }
        // The method form `OPERAND.NOUN()` reads its callee `OPERAND.NOUN`, so
        // `NOUN of OPERAND` reads what that member read does: the class
        // property `NOUN` for `NOUN of $Class`, otherwise the operand's reads.
        Expr::Of { noun, operand } => {
            member_properties(operand, noun.as_str(), class, anchors, reads);
        }
        Expr::Index { object, index } => {
            if let (Expr::ClassRef(name), Expr::String(property)) =
                (object.as_ref(), index.as_ref())
            {
                if name == class {
                    reads.live.push(property.clone());
                }
            }
            visit_children(expression, &mut |child| {
                class_properties(child, class, anchors, reads);
            });
        }
        _ => visit_children(expression, &mut |child| {
            class_properties(child, class, anchors, reads);
        }),
    }
}

/// The reads of `OBJECT.PROPERTY`: the class property itself when `OBJECT`
/// is `$Class`, otherwise whatever `OBJECT` reads.
fn member_properties(
    object: &Expr,
    property: &str,
    class: &str,
    anchors: bool,
    reads: &mut ClassReads,
) {
    if matches!(object, Expr::ClassRef(name) if name == class) {
        reads.live.push(property.to_string());
    } else {
        class_properties(object, class, anchors, reads);
    }
}

/// The property a version reference names directly on `$Class`, as
/// `$Class.property` or `$Class["property"]`.
fn class_property<'e>(expression: &'e Expr, class: &str) -> Option<&'e str> {
    let (object, property) = match expression {
        Expr::Member { object, property } if property != "value" => (object, property),
        Expr::Index { object, index } => match index.as_ref() {
            Expr::String(property) => (object, property),
            _ => return None,
        },
        _ => return None,
    };
    matches!(object.as_ref(), Expr::ClassRef(name) if name == class).then_some(property.as_str())
}

/// Visits immediate expression children without entering a function's scope.
fn visit_children(expression: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expression {
        Expr::Member { object, .. } => visit(object),
        Expr::Version { reference, .. } => visit(reference),
        Expr::Reason { source, .. } => visit(source),
        Expr::Depth { source, limit } => {
            visit(source);
            visit(limit);
        }
        Expr::Index { object, index } => {
            visit(object);
            visit(index);
        }
        Expr::Slice { object, start, end } => {
            visit(object);
            for bound in start.iter().chain(end) {
                visit(bound);
            }
        }
        Expr::Call { callee, arguments } => {
            visit(callee);
            visit_all(arguments, visit);
        }
        Expr::List(items) | Expr::Super(items) => visit_all(items, visit),
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                visit(value);
            }
        }
        Expr::SpreadObject(entries) => {
            for entry in entries {
                visit(entry.value());
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(embedded) = part {
                    visit(embedded);
                }
            }
        }
        Expr::Unary { .. }
        | Expr::Delete(_)
        | Expr::Of { .. }
        | Expr::Binary { .. }
        | Expr::Logical { .. }
        | Expr::Assign { .. }
        | Expr::Conditional { .. }
        | Expr::Quantified { .. } => visit_operands(expression, visit),
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

fn visit_operands(expression: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expression {
        Expr::Unary { operand, .. } | Expr::Delete(operand) | Expr::Of { operand, .. } => {
            visit(operand);
        }
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Assign {
            target: left,
            value: right,
        } => {
            visit(left);
            visit(right);
        }
        // Static queries see every branch, as with a skipped logical operand;
        // evaluation reads only the selected one.
        Expr::Conditional {
            condition,
            consequent,
            alternate,
        } => {
            visit(condition);
            visit(consequent);
            visit(alternate);
        }
        Expr::Quantified {
            collection,
            condition,
            ..
        } => {
            visit(collection);
            visit(condition);
        }
        Expr::Member { .. }
        | Expr::Version { .. }
        | Expr::Reason { .. }
        | Expr::Depth { .. }
        | Expr::Index { .. }
        | Expr::Slice { .. }
        | Expr::Call { .. }
        | Expr::List(_)
        | Expr::Super(_)
        | Expr::ObjectLiteral(_)
        | Expr::SpreadObject(_)
        | Expr::Template(_)
        | Expr::Null
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

fn visit_all(expressions: &[Expr], visit: &mut impl FnMut(&Expr)) {
    for expression in expressions {
        visit(expression);
    }
}
