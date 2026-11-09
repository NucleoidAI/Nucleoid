use std::fmt;

use super::words::branch_words;
use super::{
    join, write_binary, write_function, write_logical, write_object_literal, write_of,
    write_spread_object, write_template, write_unary,
};
use crate::lang::ast::{BinaryOp, Expr, LogicalOp, Quantifier, VersionSelector};
use crate::reasoning::Stage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Items {
    Words,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Plain,
    Region(Items),
    Position(Items),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assignments {
    Labels,
    Expressions,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Shown<'a> {
    pub(super) expression: &'a Expr,
    pub(super) mode: Mode,
}

impl<'a> Shown<'a> {
    pub(super) fn plain(expression: &'a Expr) -> Self {
        Shown {
            expression,
            mode: Mode::Plain,
        }
    }

    pub(super) fn inner(self, expression: &'a Expr) -> Self {
        let mode = match self.mode {
            Mode::Plain => Mode::Plain,
            Mode::Region(items) | Mode::Position(items) => Mode::Region(items),
        };
        Shown { expression, mode }
    }

    pub(super) fn item(self, expression: &'a Expr) -> Self {
        match self.mode {
            Mode::Region(Items::Words) | Mode::Position(Items::Words) => Shown {
                expression,
                mode: Mode::Position(Items::Words),
            },
            Mode::Plain | Mode::Region(Items::Plain) | Mode::Position(Items::Plain) => {
                self.inner(expression)
            }
        }
    }

    fn grouped(self, expression: &'a Expr) -> Grouped<'a> {
        Grouped(self.inner(expression))
    }

    fn arguments(
        self,
        f: &mut fmt::Formatter<'_>,
        arguments: &'a [Expr],
        assignments: Assignments,
    ) -> fmt::Result {
        for (index, argument) in arguments.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            match argument {
                Expr::Assign { target, value }
                    if assignments == Assignments::Labels
                        && matches!(target.as_ref(), Expr::Identifier(_)) =>
                {
                    write!(f, "{target}={}", self.item(value))?;
                }
                argument => write!(f, "{}", self.inner(argument))?,
            }
        }
        Ok(())
    }

    fn name(self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
        match self.mode {
            Mode::Position(_) => write!(f, "({name})"),
            Mode::Plain | Mode::Region(_) => f.write_str(name),
        }
    }

    fn version(
        self,
        f: &mut fmt::Formatter<'_>,
        reference: &'a Expr,
        selector: VersionSelector,
    ) -> fmt::Result {
        write!(f, "{}@{selector}", self.grouped(reference))
    }

    fn index(self, f: &mut fmt::Formatter<'_>, object: &'a Expr, index: &'a Expr) -> fmt::Result {
        write!(f, "{}[{}]", self.grouped(object), self.inner(index))
    }

    fn slice(
        self,
        f: &mut fmt::Formatter<'_>,
        object: &'a Expr,
        start: Option<&'a Expr>,
        end: Option<&'a Expr>,
    ) -> fmt::Result {
        write!(f, "{}[", self.grouped(object))?;
        if let Some(start) = start {
            write!(f, "{}", self.inner(start))?;
        }
        f.write_str(":")?;
        if let Some(end) = end {
            write!(f, "{}", self.inner(end))?;
        }
        f.write_str("]")
    }

    fn super_call(self, f: &mut fmt::Formatter<'_>, arguments: &'a [Expr]) -> fmt::Result {
        f.write_str("super(")?;
        self.arguments(f, arguments, Assignments::Labels)?;
        f.write_str(")")
    }

    fn call(
        self,
        f: &mut fmt::Formatter<'_>,
        callee: &'a Expr,
        arguments: &'a [Expr],
    ) -> fmt::Result {
        let assignments = match callee {
            Expr::Identifier(_) => Assignments::Labels,
            _ => Assignments::Expressions,
        };
        write!(f, "{}(", self.grouped(callee))?;
        self.arguments(f, arguments, assignments)?;
        f.write_str(")")
    }

    fn binary(
        self,
        f: &mut fmt::Formatter<'_>,
        operator: BinaryOp,
        left: &'a Expr,
        right: &'a Expr,
    ) -> fmt::Result {
        write_binary(f, operator, self.inner(left), self.inner(right))
    }

    fn logical(
        self,
        f: &mut fmt::Formatter<'_>,
        operator: LogicalOp,
        left: &'a Expr,
        right: &'a Expr,
    ) -> fmt::Result {
        write_logical(f, operator, self.inner(left), self.inner(right))
    }

    fn list(self, f: &mut fmt::Formatter<'_>, items: &'a [Expr]) -> fmt::Result {
        f.write_str("[")?;
        fmt::Display::fmt(&join(items.iter().map(|item| self.item(item))), f)?;
        f.write_str("]")
    }

    fn reason(self, f: &mut fmt::Formatter<'_>, stage: Stage, source: &'a Expr) -> fmt::Result {
        write!(f, "{}|>{}", self.inner(source), stage.as_str())
    }

    fn stage(
        self,
        f: &mut fmt::Formatter<'_>,
        source: &'a Expr,
        stage: &str,
        argument: &'a Expr,
    ) -> fmt::Result {
        write!(
            f,
            "{}|>{stage}({})",
            self.inner(source),
            self.inner(argument)
        )
    }

    fn quantified(
        self,
        f: &mut fmt::Formatter<'_>,
        quantifier: Quantifier,
        binder: &str,
        collection: &'a Expr,
        condition: &'a Expr,
    ) -> fmt::Result {
        write!(
            f,
            "({} {binder} in {}:{})",
            quantifier.as_str(),
            self.inner(collection),
            self.inner(condition)
        )
    }

    fn conditional(
        self,
        f: &mut fmt::Formatter<'_>,
        condition: &'a Expr,
        consequent: &'a Expr,
        alternate: &'a Expr,
    ) -> fmt::Result {
        match self.mode {
            Mode::Position(items) if branch_words(self.expression) => write!(
                f,
                "{}?{}:{}",
                Shown {
                    expression: condition,
                    mode: Mode::Region(items),
                },
                Shown {
                    expression: consequent,
                    mode: self.mode,
                },
                Shown {
                    expression: alternate,
                    mode: self.mode,
                }
            ),
            Mode::Plain | Mode::Region(_) | Mode::Position(_) => write!(
                f,
                "({}?{}:{})",
                self.inner(condition),
                self.inner(consequent),
                self.inner(alternate)
            ),
        }
    }
}

impl fmt::Display for Shown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = *self;
        match self.expression {
            Expr::Null => f.write_str("null"),
            Expr::Bool(bool) => write!(f, "{bool}"),
            Expr::Number(number) => f.write_str(&crate::value::format_number(*number)),
            Expr::String(string) => write!(f, "\"{string}\""),
            Expr::Template(parts) => write_template(f, parts),
            Expr::Regex(pattern) => write!(f, "/{pattern}/"),
            Expr::Identifier(name) => shown.name(f, name),
            Expr::ClassRef(name) => write!(f, "${name}"),
            Expr::ObjectRef(id) => f.write_str(id),
            Expr::This => f.write_str("this"),
            Expr::Super(arguments) => shown.super_call(f, arguments),
            Expr::Member { object, property } => write!(f, "{}.{property}", shown.grouped(object)),
            Expr::Version {
                reference,
                selector,
            } => shown.version(f, reference, *selector),
            Expr::Index { object, index } => shown.index(f, object, index),
            Expr::Slice { object, start, end } => {
                shown.slice(f, object, start.as_deref(), end.as_deref())
            }
            Expr::Call { callee, arguments } => shown.call(f, callee, arguments),
            Expr::Unary { operator, operand } => write_unary(f, *operator, shown.inner(operand)),
            Expr::Binary {
                operator,
                left,
                right,
            } => shown.binary(f, *operator, left, right),
            Expr::Logical {
                operator,
                left,
                right,
            } => shown.logical(f, *operator, left, right),
            // Both spellings render as one grouped symbol form. The grouping
            // keeps a conditional operand distinct from its enclosing operator
            // in graph keys, as `(c?a:b)+1` and `(c?a:b+1)` are different.
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } => shown.conditional(f, condition, consequent, alternate),
            Expr::List(items) => shown.list(f, items),
            Expr::ObjectLiteral(entries) => write_object_literal(f, shown, entries),
            Expr::SpreadObject(entries) => write_spread_object(f, shown, entries),
            Expr::Function(function) => write_function(f, function),
            Expr::Assign { target, value } => write!(f, "{target}={value}"),
            Expr::Delete(operand) => write!(f, "delete {}", shown.inner(operand)),
            Expr::Reason { stage, source } => shown.reason(f, *stage, source),
            Expr::Depth { source, limit } => shown.stage(f, source, "depth", limit),
            Expr::Path { source, target } => shown.stage(f, source, "path", target),
            Expr::Model => f.write_str("model"),
            Expr::Quantified {
                quantifier,
                binder,
                collection,
                condition,
            } => shown.quantified(f, *quantifier, binder, collection, condition),
            Expr::Of { noun, operand } => write_of(f, noun.as_str(), shown.inner(operand)),
            Expr::Enum(members) => write!(f, "enum{{{}}}", join(members)),
        }
    }
}

/// A postfix chain's base or a whole template interpolation. A
/// `NOUN of OPERAND` query is grouped there: its operand would otherwise take
/// the chain, as in `(mode of words).length`, and an interpolation reads only
/// a leading noun. Every other expression renders as before.
pub(super) struct Grouped<'a>(pub(super) Shown<'a>);

impl fmt::Display for Grouped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.expression {
            Expr::Of { .. } => write!(f, "({})", self.0),
            _ => write!(f, "{}", self.0),
        }
    }
}
