//! Rendering the AST back to source.
//!
//! Statement keys in the dependency graph are derived from this, so two
//! spellings of the same condition collapse onto one node.

mod shown;
mod statement;
mod words;

use std::fmt;

use crate::lang::ast::{
    BinaryOp, EnumMember, Expr, Function, FunctionBody, LogicalOp, ObjectEntry, Parameter,
    ParameterDefault, TemplatePart, UnaryOp, VersionSelector,
};
use shown::{Grouped, Shown};
pub use statement::{generate, generate_all};
use statement::{indent, source_all};
pub(crate) use words::WordValue;

fn has_binding_metadata(parameters: &[Parameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.default.is_some() || parameter.rest)
}

fn signature(parameters: &[Parameter]) -> impl fmt::Display {
    join(parameters)
}

impl fmt::Display for VersionSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(index) => write!(f, "{index}"),
            Self::First => f.write_str("first"),
            Self::Last => f.write_str("last"),
            Self::Previous => f.write_str("previous"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&Shown::plain(self), f)
    }
}

impl fmt::Display for EnumMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            ParameterDefault::String(value) if *value == self.name => f.write_str(&self.name),
            _ => write!(f, "{}={}", self.name, self.value),
        }
    }
}

pub(crate) fn enum_source(name: &str, members: &[EnumMember]) -> String {
    match members {
        [member] => format!("enum {name}:\n{}", indent(&member.to_string())),
        _ => format!("enum {name}:{}", join(members)),
    }
}

fn write_logical(
    f: &mut fmt::Formatter<'_>,
    operator: LogicalOp,
    left: Shown<'_>,
    right: Shown<'_>,
) -> fmt::Result {
    match operator {
        LogicalOp::And => write!(f, "{left}&&{right}"),
        LogicalOp::Or => write!(f, "{left}||{right}"),
        LogicalOp::Coalesce => write!(f, "({left}??{right})"),
    }
}

fn write_of(f: &mut fmt::Formatter<'_>, noun: &str, operand: Shown<'_>) -> fmt::Result {
    // The written form. Its operand is parsed as the exponent of `**`
    // is, by `unary`, so it is grouped as that exponent is.
    write!(f, "{noun} of ")?;
    power_operand(f, operand, groups_power_exponent(operand.expression))
}

fn write_template(f: &mut fmt::Formatter<'_>, parts: &[TemplatePart]) -> fmt::Result {
    f.write_str("`")?;
    for part in parts {
        match part {
            TemplatePart::Literal(literal) => f.write_str(literal)?,
            TemplatePart::Expression(expression) => {
                write!(f, "${{{}}}", Grouped(Shown::plain(expression)))?;
            }
        }
    }
    f.write_str("`")
}

fn write_unary(f: &mut fmt::Formatter<'_>, operator: UnaryOp, operand: Shown<'_>) -> fmt::Result {
    match operator {
        UnaryOp::Not => write!(f, "!{operand}"),
        UnaryOp::Negate => write!(f, "-{operand}"),
        UnaryOp::Plus => write!(f, "+{operand}"),
        UnaryOp::TypeOf => write!(f, "typeof {operand}"),
        UnaryOp::Positional => write!(f, "({operand})"),
        UnaryOp::Word => write!(f, "{operand}"),
    }
}

fn write_binary(
    f: &mut fmt::Formatter<'_>,
    operator: BinaryOp,
    left: Shown<'_>,
    right: Shown<'_>,
) -> fmt::Result {
    match operator {
        BinaryOp::Power => {
            power_operand(f, left, groups_power_base(left.expression))?;
            f.write_str(BinaryOp::Power.as_str())?;
            power_operand(f, right, groups_power_exponent(right.expression))
        }
        BinaryOp::Is | BinaryOp::IsNot | BinaryOp::In | BinaryOp::NotIn => {
            write!(f, "({left} {} {right})", operator.as_str())
        }
        _ => write!(f, "{left}{}{right}", operator.as_str()),
    }
}

fn write_object_literal(
    f: &mut fmt::Formatter<'_>,
    shown: Shown<'_>,
    entries: &[(String, Expr)],
) -> fmt::Result {
    f.write_str("{")?;
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            f.write_str(",")?;
        }
        write!(f, "\"{key}\":{}", shown.item(value))?;
    }
    f.write_str("}")
}

fn write_spread_object(
    f: &mut fmt::Formatter<'_>,
    shown: Shown<'_>,
    entries: &[ObjectEntry],
) -> fmt::Result {
    f.write_str("{")?;
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            f.write_str(",")?;
        }
        match entry {
            ObjectEntry::Property(key, value) => write!(f, "\"{key}\":{}", shown.item(value))?,
            ObjectEntry::Spread(source) => write!(f, "...{}", shown.inner(source))?,
        }
    }
    f.write_str("}")
}

fn write_function(f: &mut fmt::Formatter<'_>, function: &Function) -> fmt::Result {
    if let Some(path) = function.selector_path() {
        return write!(f, "{path}");
    }
    if !has_binding_metadata(&function.parameters) {
        return match &function.name {
            Some(name) => write!(f, "{name}()"),
            None => f.write_str("function()"),
        };
    }
    let parameters = signature(&function.parameters);
    match &function.body {
        FunctionBody::Expression(body) => write!(f, "({parameters})=>{body}"),
        FunctionBody::Block(body) => {
            write!(
                f,
                "function({parameters}){{\n{}\n}}",
                indent(&source_all(body))
            )
        }
    }
}

fn power_operand(f: &mut fmt::Formatter<'_>, operand: Shown<'_>, grouped: bool) -> fmt::Result {
    if grouped {
        write!(f, "({operand})")
    } else {
        write!(f, "{operand}")
    }
}

fn groups_power_base(operand: &Expr) -> bool {
    match operand {
        Expr::Number(number) => *number < 0.0,
        Expr::Unary { operator, .. } => *operator != UnaryOp::Positional,
        Expr::Binary {
            operator: BinaryOp::Is | BinaryOp::IsNot | BinaryOp::In | BinaryOp::NotIn,
            ..
        } => false,
        Expr::Binary { .. } | Expr::Delete(_) | Expr::Of { .. } => true,
        _ => groups_power_exponent(operand),
    }
}

fn groups_power_exponent(operand: &Expr) -> bool {
    match operand {
        Expr::Binary { operator, .. } => !matches!(
            operator,
            BinaryOp::Power | BinaryOp::Is | BinaryOp::IsNot | BinaryOp::In | BinaryOp::NotIn
        ),
        Expr::Logical { operator, .. } => *operator != LogicalOp::Coalesce,
        Expr::Assign { .. }
        | Expr::Reason { .. }
        | Expr::Depth { .. }
        | Expr::Path { .. }
        | Expr::Function(_) => true,
        _ => false,
    }
}

fn join<I>(items: I) -> Joined<I::IntoIter>
where
    I: IntoIterator<IntoIter: Clone, Item: fmt::Display>,
{
    Joined(items.into_iter())
}

struct Joined<I>(I);

impl<I> fmt::Display for Joined<I>
where
    I: Iterator<Item: fmt::Display> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, item) in self.0.clone().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            fmt::Display::fmt(&item, f)?;
        }
        Ok(())
    }
}
