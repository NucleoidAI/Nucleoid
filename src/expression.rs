//! Questions asked of a whole expression tree, rather than of one node.
//!
//! Each query visits the children relevant to its question.
//! [`Expression::roots`] stops at a function body because a lambda's parameters
//! are not names the surrounding declaration reads.

use crate::lang::ast::{Expr, FunctionBody, TemplatePart};

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
    pub fn roots(&self) -> Vec<String> {
        let mut found = Vec::new();
        roots(self.node, &mut found);
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
        class_properties(self.node, class, &mut found);
        found
    }
}

fn roots(expression: &Expr, found: &mut Vec<String>) {
    match expression {
        Expr::Identifier(name) => found.push(name.clone()),
        _ => visit_children(expression, &mut |child| roots(child, found)),
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

fn class_properties(expression: &Expr, class: &str, found: &mut Vec<String>) {
    match expression {
        Expr::Member { property, .. } if property == "value" => {}
        Expr::Member { object, property } => {
            if matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
                found.push(property.clone());
            } else {
                class_properties(object, class, found);
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
                class_properties(child, class, found)
            });
        }
        _ => visit_children(expression, &mut |child| {
            class_properties(child, class, found)
        }),
    }
}

/// Visits immediate expression children without entering a function's scope.
fn visit_children(expression: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expression {
        Expr::Member { object, .. } => visit(object),
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
