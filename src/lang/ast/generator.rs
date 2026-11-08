//! Rendering the AST back to source.
//!
//! Statement keys in the dependency graph are derived from this, so two
//! spellings of the same condition collapse onto one node.

mod statement;

use std::fmt;

use crate::lang::ast::{
    BinaryOp, EnumMember, Expr, Function, FunctionBody, LogicalOp, ObjectEntry, Parameter,
    ParameterDefault, Quantifier, TemplatePart, UnaryOp, VersionSelector,
};
pub use statement::{generate, generate_all};
use statement::{indent, source_all};

fn has_binding_metadata(parameters: &[Parameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.default.is_some() || parameter.rest)
}

fn signature(parameters: &[Parameter]) -> String {
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
        match self {
            Expr::Null => f.write_str("null"),
            Expr::Bool(bool) => write!(f, "{bool}"),
            Expr::Number(number) => f.write_str(&crate::value::format_number(*number)),
            Expr::String(string) => write!(f, "\"{string}\""),
            Expr::Template(parts) => write_template(f, parts),
            Expr::Regex(pattern) => write!(f, "/{pattern}/"),
            Expr::Identifier(name) => f.write_str(name),
            Expr::ClassRef(name) => write!(f, "${name}"),
            Expr::ObjectRef(id) => f.write_str(id),
            Expr::This => f.write_str("this"),
            Expr::Super(arguments) => write!(f, "super({})", join(arguments)),
            Expr::Member { object, property } => write!(f, "{}.{property}", Grouped(object)),
            Expr::Version {
                reference,
                selector,
            } => write!(f, "{}@{selector}", Grouped(reference)),
            Expr::Index { object, index } => write!(f, "{}[{index}]", Grouped(object)),
            Expr::Slice { object, start, end } => {
                write_slice(f, object, start.as_deref(), end.as_deref())
            }
            Expr::Call { callee, arguments } => {
                write!(f, "{}({})", Grouped(callee), join(arguments))
            }
            Expr::Unary { operator, operand } => write_unary(f, *operator, operand),
            Expr::Binary {
                operator,
                left,
                right,
            } => write_binary(f, *operator, left, right),
            Expr::Logical {
                operator,
                left,
                right,
            } => write_logical(f, *operator, left, right),
            // Both spellings render as one grouped symbol form. The grouping
            // keeps a conditional operand distinct from its enclosing operator
            // in graph keys, as `(c?a:b)+1` and `(c?a:b+1)` are different.
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } => write!(f, "({condition}?{consequent}:{alternate})"),
            Expr::List(items) => write!(f, "[{}]", join(items)),
            Expr::ObjectLiteral(entries) => write_object_literal(f, entries),
            Expr::SpreadObject(entries) => write_spread_object(f, entries),
            Expr::Function(function) => write_function(f, function),
            Expr::Assign { target, value } => write!(f, "{target}={value}"),
            Expr::Delete(operand) => write!(f, "delete {operand}"),
            Expr::Reason { stage, source } => write!(f, "{source}|>{}", stage.as_str()),
            Expr::Depth { source, limit } => write!(f, "{source}|>depth({limit})"),
            Expr::Path { source, target } => write!(f, "{source}|>path({target})"),
            Expr::Model => f.write_str("model"),
            Expr::Quantified {
                quantifier,
                binder,
                collection,
                condition,
            } => write_quantified(f, *quantifier, binder, collection, condition),
            Expr::Of { noun, operand } => write_of(f, noun.as_str(), operand),
            Expr::Enum(members) => write!(f, "enum{{{}}}", join(members)),
        }
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

fn write_quantified(
    f: &mut fmt::Formatter<'_>,
    quantifier: Quantifier,
    binder: &str,
    collection: &Expr,
    condition: &Expr,
) -> fmt::Result {
    write!(
        f,
        "({} {binder} in {collection}:{condition})",
        quantifier.as_str()
    )
}

fn write_logical(
    f: &mut fmt::Formatter<'_>,
    operator: LogicalOp,
    left: &Expr,
    right: &Expr,
) -> fmt::Result {
    match operator {
        LogicalOp::And => write!(f, "{left}&&{right}"),
        LogicalOp::Or => write!(f, "{left}||{right}"),
        LogicalOp::Coalesce => write!(f, "({left}??{right})"),
    }
}

fn write_of(f: &mut fmt::Formatter<'_>, noun: &str, operand: &Expr) -> fmt::Result {
    // The written form. Its operand is parsed as the exponent of `**`
    // is, by `unary`, so it is grouped as that exponent is.
    write!(f, "{noun} of ")?;
    power_operand(f, operand, groups_power_exponent(operand))
}

fn write_template(f: &mut fmt::Formatter<'_>, parts: &[TemplatePart]) -> fmt::Result {
    f.write_str("`")?;
    for part in parts {
        match part {
            TemplatePart::Literal(literal) => f.write_str(literal)?,
            TemplatePart::Expression(expression) => {
                write!(f, "${{{}}}", Grouped(expression))?;
            }
        }
    }
    f.write_str("`")
}

fn write_slice(
    f: &mut fmt::Formatter<'_>,
    object: &Expr,
    start: Option<&Expr>,
    end: Option<&Expr>,
) -> fmt::Result {
    write!(f, "{}[", Grouped(object))?;
    if let Some(start) = start {
        write!(f, "{start}")?;
    }
    f.write_str(":")?;
    if let Some(end) = end {
        write!(f, "{end}")?;
    }
    f.write_str("]")
}

fn write_unary(f: &mut fmt::Formatter<'_>, operator: UnaryOp, operand: &Expr) -> fmt::Result {
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
    left: &Expr,
    right: &Expr,
) -> fmt::Result {
    match operator {
        BinaryOp::Power => {
            power_operand(f, left, groups_power_base(left))?;
            f.write_str(BinaryOp::Power.as_str())?;
            power_operand(f, right, groups_power_exponent(right))
        }
        BinaryOp::Is | BinaryOp::IsNot => write!(f, "({left} {} {right})", operator.as_str()),
        _ => write!(f, "{left}{}{right}", operator.as_str()),
    }
}

fn write_object_literal(f: &mut fmt::Formatter<'_>, entries: &[(String, Expr)]) -> fmt::Result {
    let rendered: Vec<String> = entries
        .iter()
        .map(|(key, value)| format!("\"{key}\":{value}"))
        .collect();
    write!(f, "{{{}}}", rendered.join(","))
}

fn write_spread_object(f: &mut fmt::Formatter<'_>, entries: &[ObjectEntry]) -> fmt::Result {
    f.write_str("{")?;
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            f.write_str(",")?;
        }
        match entry {
            ObjectEntry::Property(key, value) => write!(f, "\"{key}\":{value}")?,
            ObjectEntry::Spread(source) => write!(f, "...{source}")?,
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

/// A postfix chain's base or a whole template interpolation. A
/// `NOUN of OPERAND` query is grouped there: its operand would otherwise take
/// the chain, as in `(mode of words).length`, and an interpolation reads only
/// a leading noun. Every other expression renders as before.
struct Grouped<'a>(&'a Expr);

impl fmt::Display for Grouped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Expr::Of { .. } => write!(f, "({})", self.0),
            expression => write!(f, "{expression}"),
        }
    }
}

fn power_operand(f: &mut fmt::Formatter<'_>, operand: &Expr, grouped: bool) -> fmt::Result {
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
            operator: BinaryOp::Is | BinaryOp::IsNot,
            ..
        } => false,
        Expr::Binary { .. } | Expr::Delete(_) | Expr::Of { .. } => true,
        _ => groups_power_exponent(operand),
    }
}

fn groups_power_exponent(operand: &Expr) -> bool {
    match operand {
        Expr::Binary { operator, .. } => {
            !matches!(operator, BinaryOp::Power | BinaryOp::Is | BinaryOp::IsNot)
        }
        Expr::Logical { operator, .. } => *operator != LogicalOp::Coalesce,
        Expr::Assign { .. }
        | Expr::Reason { .. }
        | Expr::Depth { .. }
        | Expr::Path { .. }
        | Expr::Function(_) => true,
        _ => false,
    }
}

fn join<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
