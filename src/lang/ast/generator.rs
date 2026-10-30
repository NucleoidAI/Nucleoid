//! Rendering the AST back to source.
//!
//! Statement keys in the dependency graph are derived from this, so two
//! spellings of the same condition collapse onto one node.

use std::fmt;

use crate::lang::ast::{
    ClassDecl, Expr, Function, FunctionBody, LogicalOp, Parameter, Stmt, TemplatePart, UnaryOp,
    VersionSelector,
};

fn has_defaults(parameters: &[Parameter]) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.default.is_some())
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
            Expr::Template(parts) => {
                f.write_str("`")?;
                for part in parts {
                    match part {
                        TemplatePart::Literal(literal) => f.write_str(literal)?,
                        TemplatePart::Expression(expression) => write!(f, "${{{expression}}}")?,
                    }
                }
                f.write_str("`")
            }
            Expr::Regex(pattern) => write!(f, "/{pattern}/"),
            Expr::Identifier(name) => f.write_str(name),
            Expr::ClassRef(name) => write!(f, "${name}"),
            Expr::ObjectRef(id) => f.write_str(id),
            Expr::This => f.write_str("this"),
            Expr::Super(arguments) => {
                write!(f, "super({})", join(arguments))
            }
            Expr::Member { object, property } => write!(f, "{object}.{property}"),
            Expr::Version {
                reference,
                selector,
            } => write!(f, "{reference}@{selector}"),
            Expr::Index { object, index } => write!(f, "{object}[{index}]"),
            Expr::Slice { object, start, end } => {
                write!(f, "{object}[")?;
                if let Some(start) = start {
                    write!(f, "{start}")?;
                }
                f.write_str(":")?;
                if let Some(end) = end {
                    write!(f, "{end}")?;
                }
                f.write_str("]")
            }
            Expr::Call { callee, arguments } => write!(f, "{callee}({})", join(arguments)),
            Expr::Unary { operator, operand } => match operator {
                UnaryOp::Not => write!(f, "!{operand}"),
                UnaryOp::Negate => write!(f, "-{operand}"),
                UnaryOp::Plus => write!(f, "+{operand}"),
                UnaryOp::TypeOf => write!(f, "typeof {operand}"),
                UnaryOp::Positional => write!(f, "({operand})"),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => write!(f, "{left}{}{right}", operator.as_str()),
            Expr::Logical {
                operator,
                left,
                right,
            } => match operator {
                LogicalOp::And => write!(f, "{left}&&{right}"),
                LogicalOp::Or => write!(f, "{left}||{right}"),
                LogicalOp::Coalesce => write!(f, "({left}??{right})"),
            },
            // Both spellings render as one grouped symbol form. The grouping
            // keeps a conditional operand distinct from its enclosing operator
            // in graph keys, as `(c?a:b)+1` and `(c?a:b+1)` are different.
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } => write!(f, "({condition}?{consequent}:{alternate})"),
            Expr::List(items) => write!(f, "[{}]", join(items)),
            Expr::ObjectLiteral(entries) => {
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("\"{key}\":{value}"))
                    .collect();
                write!(f, "{{{}}}", rendered.join(","))
            }
            Expr::Function(function) if has_defaults(&function.parameters) => {
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
            Expr::Function(function) => match &function.name {
                Some(name) => write!(f, "{name}()"),
                None => f.write_str("function()"),
            },
            Expr::Assign { target, value } => write!(f, "{target}={value}"),
            Expr::Delete(operand) => write!(f, "delete {operand}"),
            Expr::Reason { stage, source } => write!(f, "{source}|>{}", stage.as_str()),
            Expr::Model => f.write_str("model"),
        }
    }
}

fn join(expressions: &[Expr]) -> String {
    expressions
        .iter()
        .map(|expression| expression.to_string())
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

pub fn generate(statement: &Stmt) -> String {
    match statement {
        Stmt::Assign { target, value } => format!("{target}={value}"),
        Stmt::Expression(expression) => expression.to_string(),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            let alternate = alternate
                .as_ref()
                .map(|statement| format!("else{{{}}}", generate(statement)))
                .unwrap_or_default();
            format!("if({condition}){{{}}}{alternate}", generate_all(consequent))
        }
        Stmt::Block(statements) => format!("{{{}}}", generate_all(statements)),
        Stmt::Return(Some(expression)) => format!("return {expression}"),
        Stmt::Return(None) => "return".to_string(),
        Stmt::Throw(expression) => format!("throw {expression}"),
        Stmt::Delete(expression) => format!("delete {expression}"),
        Stmt::Class(declaration)
            if has_defaults(&declaration.parameters)
                || declaration
                    .methods
                    .iter()
                    .any(|method| has_defaults(&method.parameters)) =>
        {
            class_source(declaration)
        }
        Stmt::Class(declaration) => format!("class {}", declaration.name),
        Stmt::Function(function) if has_defaults(&function.parameters) => function_source(function),
        Stmt::Function(function) => match &function.name {
            Some(name) => format!("def {name}"),
            None => "def".to_string(),
        },
        Stmt::For {
            variable, iterable, ..
        } => format!("for {variable} of {iterable}"),
        Stmt::Try { .. } => "try".to_string(),
        Stmt::Declaration { name, type_name } => format!("{name}:{type_name}"),
        Stmt::Pass => "pass".to_string(),
    }
}

// Default-bearing declarations must not discard signature metadata. Keep the
// established compact labels for every default-free declaration; render the
// new declarations' bodies with real suite syntax, including init/method data.
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

/// Suite-aware rendering inside a new default-bearing declaration. The
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
                    statement => source(statement),
                };
                rendered.push_str(&format!("\nelse:\n{}", indent(&body)));
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
        statement => generate(statement),
    }
}
