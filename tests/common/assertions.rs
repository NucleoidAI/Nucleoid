//! Ordered canonical assertions, including nested function and template bodies.

use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, TemplatePart};

pub fn protected_assertions(statements: &[Stmt]) -> Vec<String> {
    let mut assertions = Vec::new();
    for statement in statements {
        statement_assertions(statement, &mut assertions);
    }
    assertions
}

fn statement_assertions(statement: &Stmt, assertions: &mut Vec<String>) {
    match statement {
        Stmt::Assign { target, value } => {
            expression_assertions(target, assertions);
            expression_assertions(value, assertions);
        }
        Stmt::Expression(expression)
        | Stmt::Throw(expression)
        | Stmt::Delete(expression)
        | Stmt::Return(Some(expression)) => expression_assertions(expression, assertions),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            expression_assertions(condition, assertions);
            for statement in consequent {
                statement_assertions(statement, assertions);
            }
            if let Some(statement) = alternate {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Block(body) => {
            for statement in body {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Class(class) => {
            for statement in &class.constructor {
                statement_assertions(statement, assertions);
            }
            for method in &class.methods {
                body_assertions(&method.body, assertions);
            }
        }
        Stmt::Function(function) => body_assertions(&function.body, assertions),
        Stmt::For { iterable, body, .. } => {
            expression_assertions(iterable, assertions);
            for statement in body {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Try { body, catch, .. } => {
            for statement in body.iter().chain(catch) {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Return(None) | Stmt::Declaration { .. } | Stmt::Pass => {}
    }
}

fn body_assertions(body: &FunctionBody, assertions: &mut Vec<String>) {
    match body {
        FunctionBody::Expression(expression) => expression_assertions(expression, assertions),
        FunctionBody::Block(body) => {
            for statement in body {
                statement_assertions(statement, assertions);
            }
        }
    }
}

fn expression_assertions(expression: &Expr, assertions: &mut Vec<String>) {
    match expression {
        Expr::Call { callee, arguments } => {
            if matches!(callee.as_ref(), Expr::Identifier(name) if name == "assert") {
                assertions.push(expression.to_string());
            }
            expression_assertions(callee, assertions);
            for argument in arguments {
                expression_assertions(argument, assertions);
            }
        }
        Expr::Member { object, .. } => expression_assertions(object, assertions),
        Expr::Version { reference, .. } => expression_assertions(reference, assertions),
        Expr::Index { object, index } => {
            expression_assertions(object, assertions);
            expression_assertions(index, assertions);
        }
        Expr::Slice { object, start, end } => {
            expression_assertions(object, assertions);
            for bound in start.iter().chain(end) {
                expression_assertions(bound, assertions);
            }
        }
        Expr::Unary { operand, .. } | Expr::Delete(operand) => {
            expression_assertions(operand, assertions)
        }
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Assign {
            target: left,
            value: right,
        } => {
            expression_assertions(left, assertions);
            expression_assertions(right, assertions);
        }
        Expr::Conditional {
            condition,
            consequent,
            alternate,
        } => {
            expression_assertions(condition, assertions);
            expression_assertions(consequent, assertions);
            expression_assertions(alternate, assertions);
        }
        Expr::Super(items) | Expr::List(items) => {
            for item in items {
                expression_assertions(item, assertions);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                expression_assertions(value, assertions);
            }
        }
        Expr::Function(function) => body_assertions(&function.body, assertions),
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    expression_assertions(expression, assertions);
                }
            }
        }
        Expr::Reason { source, .. } => expression_assertions(source, assertions),
        Expr::Null
        | Expr::Bool(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::Regex(_)
        | Expr::Identifier(_)
        | Expr::ClassRef(_)
        | Expr::ObjectRef(_)
        | Expr::This
        | Expr::Model => {}
    }
}
