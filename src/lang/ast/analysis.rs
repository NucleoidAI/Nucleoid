use super::{Expr, Stmt};
use crate::expression::Expression;
use crate::runtime::Runtime;
use crate::scope::Scope;

impl Expr {
    /// A bare identifier assignment can be a label when the call resolves to a
    /// constructor. Explicit positional assignments have their own boundary.
    pub(crate) fn named_argument(&self) -> Option<(&str, &Expr)> {
        if let Expr::Assign { target, value } = self {
            if let Expr::Identifier(name) = target.as_ref() {
                return Some((name, value));
            }
        }
        None
    }

    pub(crate) fn has_named_arguments(&self) -> bool {
        match self {
            Expr::Call { arguments, .. } | Expr::Super(arguments) => arguments
                .iter()
                .any(|argument| argument.named_argument().is_some()),
            _ => false,
        }
    }

    /// Renders a path for an error message, preferring the source spelling.
    pub(crate) fn describe(&self) -> String {
        self.path().unwrap_or_else(|| self.to_string())
    }

    /// The dotted path this expression denotes, when it is a plain identifier
    /// chain such as `a`, `person1.name` or `$Person.mortal`.
    #[must_use]
    pub fn path(&self) -> Option<String> {
        match self {
            Expr::Identifier(name) | Expr::ClassRef(name) | Expr::ObjectRef(name) => {
                Some(name.clone())
            }
            Expr::Member { object, property } => {
                object.path().map(|base| format!("{base}.{property}"))
            }
            _ => None,
        }
    }

    /// The leftmost node of a path expression.
    #[must_use]
    pub fn first(&self) -> Option<&Expr> {
        match self {
            Expr::Identifier(_) | Expr::ClassRef(_) | Expr::ObjectRef(_) | Expr::This => Some(self),
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => object.first(),
            Expr::Version { reference, .. } => reference.first(),
            Expr::Call { callee, .. } => callee.first(),
            Expr::Of { operand, .. } => operand.first(),
            _ => None,
        }
    }

    /// What this node is read from — the `a.b` of `a.b.c`.
    #[must_use]
    pub fn object(&self) -> Option<&Expr> {
        match self {
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => Some(object),
            Expr::Version { reference, .. } => Some(reference),
            _ => None,
        }
    }

    #[must_use]
    pub fn contains_reasoning(&self) -> bool {
        match self {
            Expr::Reason { .. } | Expr::Depth { .. } | Expr::Path { .. } | Expr::Model => true,
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => object.contains_reasoning(),
            Expr::Version { reference, .. } => reference.contains_reasoning(),
            Expr::Unary { operand, .. } | Expr::Delete(operand) | Expr::Of { operand, .. } => {
                operand.contains_reasoning()
            }
            Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. }
            | Expr::Assign {
                target: left,
                value: right,
            } => left.contains_reasoning() || right.contains_reasoning(),
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } => {
                condition.contains_reasoning()
                    || consequent.contains_reasoning()
                    || alternate.contains_reasoning()
            }
            Expr::Quantified {
                collection,
                condition,
                ..
            } => collection.contains_reasoning() || condition.contains_reasoning(),
            Expr::Call { callee, arguments } => {
                callee.contains_reasoning() || arguments.iter().any(Expr::contains_reasoning)
            }
            Expr::List(items) => items.iter().any(Expr::contains_reasoning),
            Expr::ObjectLiteral(entries) => {
                entries.iter().any(|(_, value)| value.contains_reasoning())
            }
            Expr::SpreadObject(entries) => entries
                .iter()
                .any(|entry| entry.value().contains_reasoning()),
            _ => false,
        }
    }

    /// Whether a list expression is derived from a version read. Real object
    /// properties are still writable; callers resolve their owner first.
    pub(crate) fn version_view(&self) -> bool {
        match self {
            Expr::Version { .. } => true,
            Expr::Member { object, .. }
            | Expr::Index { object, .. }
            | Expr::Slice { object, .. } => object.version_view(),
            _ => false,
        }
    }

    /// The rightmost name of a path expression — the `c` of `a.b.c`.
    #[must_use]
    pub fn last(&self) -> Option<&str> {
        match self {
            Expr::Identifier(name) | Expr::ClassRef(name) | Expr::ObjectRef(name) => Some(name),
            Expr::Member { property, .. } => Some(property),
            _ => None,
        }
    }
}

// -- walking -----------------------------------------------------------------

/// The bare names a statement assigns to.
pub(crate) fn collect_assigned_names(statement: &Stmt, names: &mut Vec<String>) {
    match statement {
        Stmt::Assign {
            target: Expr::Identifier(name),
            ..
        }
        | Stmt::ValueAssign {
            target: Expr::Identifier(name),
            ..
        } => names.push(name.clone()),
        Stmt::Block(nested) => {
            for child in nested {
                collect_assigned_names(child, names);
            }
        }
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            for child in consequent {
                collect_assigned_names(child, names);
            }
            if let Some(alternate) = alternate {
                collect_assigned_names(alternate, names);
            }
        }
        _ => {}
    }
}

/// The leftmost names a statement reads.
pub(crate) fn collect_read_roots(
    statement: &Stmt,
    roots: &mut Vec<String>,
    runtime: &Runtime,
    scope: &Scope,
) {
    match statement {
        Stmt::Assign { target, value } | Stmt::ValueAssign { target, value } => {
            if let Expr::Member { object, .. } = target {
                roots.extend(runtime.expression_roots(object, scope, false));
            }
            roots.extend(runtime.expression_roots(
                value,
                scope,
                matches!(target, Expr::Member { .. }),
            ));
        }
        Stmt::Expression(expression)
        | Stmt::Throw(expression)
        | Stmt::Delete(expression)
        | Stmt::Return(Some(expression)) => {
            roots.extend(runtime.expression_roots(expression, scope, false));
        }
        Stmt::Block(nested) => {
            for child in nested {
                collect_read_roots(child, roots, runtime, scope);
            }
        }
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            roots.extend(runtime.expression_roots(condition, scope, false));
            for child in consequent {
                collect_read_roots(child, roots, runtime, scope);
            }
            if let Some(alternate) = alternate {
                collect_read_roots(alternate, roots, runtime, scope);
            }
        }
        _ => {}
    }
}

/// The first `$Class` a statement mentions, anywhere inside it.
pub(crate) fn find_class_reference_statement(statement: &Stmt, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match statement {
        Stmt::Assign { target, value } | Stmt::ValueAssign { target, value } => {
            take(found, target);
            take(found, value);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            take(found, expression);
        }
        Stmt::Return(Some(expression)) => take(found, expression),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            take(found, condition);
            for child in consequent {
                find_class_reference_statement(child, found);
            }
            if let Some(alternate) = alternate {
                find_class_reference_statement(alternate, found);
            }
        }
        Stmt::Block(statements) => {
            for child in statements {
                find_class_reference_statement(child, found);
            }
        }
        _ => {}
    }
}

fn take(found: &mut Option<String>, expression: &Expr) {
    if found.is_none() {
        *found = Expression::new(expression).class_reference();
    }
}
