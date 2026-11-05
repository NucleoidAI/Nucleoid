//! Rendering the AST back to source.
//!
//! Statement keys in the dependency graph are derived from this, so two
//! spellings of the same condition collapse onto one node.

use std::fmt;

use crate::lang::ast::{
    BinaryOp, ClassDecl, Expr, Function, FunctionBody, LogicalOp, ObjectEntry, Parameter, Stmt,
    TemplatePart, UnaryOp, VersionSelector,
};

fn has_binding_metadata(parameters: &[Parameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.default.is_some() || parameter.rest)
}

fn signature(parameters: &[Parameter]) -> String {
    parameters
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
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
            Expr::Model => f.write_str("model"),
            Expr::Quantified {
                quantifier,
                binder,
                collection,
                condition,
            } => write!(
                f,
                "({} {binder} in {collection}:{condition})",
                quantifier.as_str()
            ),
            Expr::Of { noun, operand } => write_of(f, noun.as_str(), operand),
        }
    }
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
    }
}

fn write_binary(
    f: &mut fmt::Formatter<'_>,
    operator: BinaryOp,
    left: &Expr,
    right: &Expr,
) -> fmt::Result {
    if operator != BinaryOp::Power {
        return write!(f, "{left}{}{right}", operator.as_str());
    }
    power_operand(f, left, groups_power_base(left))?;
    f.write_str(BinaryOp::Power.as_str())?;
    power_operand(f, right, groups_power_exponent(right))
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
        Expr::Binary { .. } | Expr::Delete(_) | Expr::Of { .. } => true,
        _ => groups_power_exponent(operand),
    }
}

fn groups_power_exponent(operand: &Expr) -> bool {
    match operand {
        Expr::Binary { operator, .. } => *operator != BinaryOp::Power,
        Expr::Logical { operator, .. } => *operator != LogicalOp::Coalesce,
        Expr::Assign { .. } | Expr::Reason { .. } | Expr::Function(_) => true,
        _ => false,
    }
}

fn join(expressions: &[Expr]) -> String {
    expressions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Renders a run of statements as the body of a block, which is what a block or
/// branch is keyed by in the graph.
pub fn generate_all(statements: &[Stmt]) -> String {
    statements
        .iter()
        .map(generate)
        .collect::<Vec<_>>()
        .join(";")
}

#[must_use]
pub fn generate(statement: &Stmt) -> String {
    match statement {
        Stmt::Assign { target, value } => format!("{target}={value}"),
        Stmt::ValueAssign { target, value } => format!("{target}:={value}"),
        Stmt::Expression(expression) => expression.to_string(),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            let alternate = alternate
                .as_ref()
                .map(|branch| format!("else{{{}}}", generate(branch)))
                .unwrap_or_default();
            format!("if({condition}){{{}}}{alternate}", generate_all(consequent))
        }
        Stmt::Block(statements) => format!("{{{}}}", generate_all(statements)),
        Stmt::Return(Some(expression)) => format!("return {expression}"),
        Stmt::Return(None) => "return".to_string(),
        Stmt::Throw(expression) => format!("throw {expression}"),
        Stmt::Delete(expression) => format!("delete {expression}"),
        Stmt::Class(declaration)
            if has_binding_metadata(&declaration.parameters)
                || declaration
                    .methods
                    .iter()
                    .any(|method| has_binding_metadata(&method.parameters)) =>
        {
            class_source(declaration)
        }
        Stmt::Class(declaration) => format!("class {}", declaration.name),
        Stmt::Function(function) if has_binding_metadata(&function.parameters) => {
            function_source(function)
        }
        Stmt::Function(function) => match &function.name {
            Some(name) => format!("def {name}"),
            None => "def".to_string(),
        },
        Stmt::For {
            variable, iterable, ..
        } => format!("for {variable} of {iterable}"),
        Stmt::ForEntries {
            key,
            value,
            iterable,
            ..
        } => format!("for {key},{value} of {iterable}"),
        Stmt::While { condition, body } => {
            format!("while({condition}){{{}}}", generate_all(body))
        }
        Stmt::Try { .. } => "try".to_string(),
        Stmt::Declaration { name, type_name } => format!("{name}:{type_name}"),
        Stmt::Pass => "pass".to_string(),
        Stmt::Suppose { target, body } => {
            format!(
                "{}suppose{{{}}}",
                suppose_target(target.as_ref()),
                generate_all(body)
            )
        }
    }
}

fn suppose_target(target: Option<&Expr>) -> String {
    target
        .map(|target| format!("{target}="))
        .unwrap_or_default()
}

// Default/rest-bearing declarations must not discard signature metadata. Keep
// established compact labels for every other declaration; render the new
// declarations' bodies with real suite syntax, including init/method data.
fn indent(source: &str) -> String {
    source
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_all(statements: &[Stmt]) -> String {
    if statements.is_empty() {
        return "pass".to_string();
    }
    statements.iter().map(source).collect::<Vec<_>>().join("\n")
}

fn function_source(function: &Function) -> String {
    let name = function.name.as_deref().unwrap_or_default();
    let header = format!("def {name}({})", signature(&function.parameters));
    match &function.body {
        FunctionBody::Expression(body) => format!("{header}=>{body}"),
        FunctionBody::Block(body) => format!("{header}:\n{}", indent(&source_all(body))),
    }
}

fn class_source(class: &ClassDecl) -> String {
    let mut header = format!("class {}({})", class.name, signature(&class.parameters));
    let mut body = class
        .fields
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    body.extend(class.constructor.iter().map(source));
    body.extend(class.methods.iter().map(|method| function_source(method)));
    if class.parent.is_none() && body.is_empty() {
        return header;
    }
    header.push(':');
    if let Some(parent) = &class.parent {
        header.push(' ');
        header.push_str(parent);
    }
    let body = if body.is_empty() {
        "pass".to_string()
    } else {
        body.join("\n")
    };
    format!("{header}\n{}", indent(&body))
}

/// Suite-aware rendering inside a new default/rest-bearing declaration. The
/// public graph-label generator deliberately abbreviates legacy declarations.
fn source(statement: &Stmt) -> String {
    match statement {
        Stmt::Function(function) => function_source(function),
        Stmt::Class(class) => class_source(class),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            let mut rendered = format!("if {condition}:\n{}", indent(&source_all(consequent)));
            if let Some(alternate) = alternate {
                let body = match alternate.as_ref() {
                    Stmt::Block(body) => source_all(body),
                    branch => source(branch),
                };
                rendered.push_str("\nelse:\n");
                rendered.push_str(&indent(&body));
            }
            rendered
        }
        Stmt::For {
            variable,
            iterable,
            body,
        } => {
            format!(
                "for {variable} of {iterable}:\n{}",
                indent(&source_all(body))
            )
        }
        Stmt::ForEntries {
            key,
            value,
            iterable,
            body,
        } => format!(
            "for {key},{value} of {iterable}:\n{}",
            indent(&source_all(body))
        ),
        Stmt::While { condition, body } => {
            format!("while {condition}:\n{}", indent(&source_all(body)))
        }
        Stmt::Try {
            body,
            parameter,
            catch,
        } => format!(
            "try:\n{}\ncatch {parameter}:\n{}",
            indent(&source_all(body)),
            indent(&source_all(catch))
        ),
        Stmt::Block(body) => format!("{{\n{}\n}}", source_all(body)),
        Stmt::Suppose { target, body } => format!(
            "{}suppose:\n{}",
            suppose_target(target.as_ref()),
            indent(&source_all(body))
        ),
        statement => generate(statement),
    }
}
