//! Questions asked of a whole expression tree, rather than of one node.
//! Mirrors `ref/src/Expression.js`, which likewise sits above
//! [`lang::ast`](crate::lang::ast) and walks a node for what is inside it.
//!
//! `ref` has one traversal — `map`, `find` and `traverse` all descend through
//! operators and hand everything below to `Node.convert` — because whatever it
//! does not understand is passed to `eval` anyway. Nothing is handed to another
//! language here, so each query walks exactly as deep as its own question
//! needs, and no deeper: [`Expression::roots`] stops at a function body on
//! purpose, since a lambda's parameters are not names the surrounding
//! declaration reads.

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
        Expr::Member { object, .. } | Expr::Slice { object, .. } => roots(object, found),
        Expr::Index { object, index } => {
            roots(object, found);
            roots(index, found);
        }
        Expr::Call { callee, arguments } => {
            roots(callee, found);
            for argument in arguments {
                roots(argument, found);
            }
        }
        Expr::Unary { operand, .. } => roots(operand, found),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            roots(left, found);
            roots(right, found);
        }
        Expr::List(items) => {
            for item in items {
                roots(item, found);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                roots(value, found);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    roots(expression, found);
                }
            }
        }
        Expr::Assign { target, value } => {
            roots(target, found);
            roots(value, found);
        }
        _ => {}
    }
}

fn class_reference(expression: &Expr, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match expression {
        Expr::ClassRef(name) => *found = Some(name.clone()),
        Expr::Member { object, .. } => class_reference(object, found),
        Expr::Index { object, index } => {
            class_reference(object, found);
            class_reference(index, found);
        }
        Expr::Slice { object, start, end } => {
            class_reference(object, found);
            if let Some(start) = start {
                class_reference(start, found);
            }
            if let Some(end) = end {
                class_reference(end, found);
            }
        }
        Expr::Call { callee, arguments } => {
            class_reference(callee, found);
            for argument in arguments {
                class_reference(argument, found);
            }
        }
        Expr::Unary { operand, .. } => class_reference(operand, found),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            class_reference(left, found);
            class_reference(right, found);
        }
        Expr::List(items) => {
            for item in items {
                class_reference(item, found);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                class_reference(value, found);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    class_reference(expression, found);
                }
            }
        }
        Expr::Assign { target, value } => {
            class_reference(target, found);
            class_reference(value, found);
        }
        // A rule written as a lambda still states something about the type, so
        // unlike `roots` this does look inside one.
        Expr::Function(function) => {
            if let FunctionBody::Expression(body) = &function.body {
                class_reference(body, found);
            }
        }
        _ => {}
    }
}

fn class_properties(expression: &Expr, class: &str, found: &mut Vec<String>) {
    match expression {
        Expr::Member { object, property } => {
            if matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
                found.push(property.clone());
            } else {
                class_properties(object, class, found);
            }
        }
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            class_properties(left, class, found);
            class_properties(right, class, found);
        }
        Expr::Unary { operand, .. } => class_properties(operand, class, found),
        Expr::Call { callee, arguments } => {
            class_properties(callee, class, found);
            for argument in arguments {
                class_properties(argument, class, found);
            }
        }
        Expr::Index { object, index } => {
            class_properties(object, class, found);
            class_properties(index, class, found);
        }
        _ => {}
    }
}
