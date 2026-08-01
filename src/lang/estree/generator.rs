//! Rendering the AST back to source.
//!
//! Statement keys in the dependency graph are derived from this, so two
//! spellings of the same condition collapse onto one node.

use std::fmt;

use crate::lang::ast::{Expr, LogicalOp, Stmt, TemplatePart, UnaryOp};

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
            } => {
                let operator = match operator {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                };
                write!(f, "{left}{operator}{right}")
            }
            Expr::List(items) => write!(f, "[{}]", join(items)),
            Expr::ObjectLiteral(entries) => {
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("\"{key}\":{value}"))
                    .collect();
                write!(f, "{{{}}}", rendered.join(","))
            }
            Expr::Function(function) => match &function.name {
                Some(name) => write!(f, "{name}()"),
                None => f.write_str("function()"),
            },
            Expr::Assign { target, value } => write!(f, "{target}={value}"),
            Expr::Delete(operand) => write!(f, "delete {operand}"),
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
        Stmt::Class(declaration) => format!("class {}", declaration.name),
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
