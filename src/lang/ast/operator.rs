//! Unary, binary, logical and conditional operators.

use std::cmp::Ordering;

use crate::error::{Error, Result};
use crate::lang::ast::{BinaryOp, Expr, LogicalOp, UnaryOp};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug)]
pub struct Operator<'a> {
    pub node: &'a Expr,
}

impl<'a> Operator<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Operator { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::Unary { operator, operand } => runtime.evaluate_unary(*operator, operand, scope),
            Expr::Logical {
                operator,
                left,
                right,
            } => runtime.evaluate_logical(*operator, left, right, scope),
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } => runtime.evaluate_conditional(condition, consequent, alternate, scope),
            Expr::Binary {
                operator,
                left,
                right,
            } => runtime.evaluate_binary(*operator, left, right, scope),
            Expr::Assign { target, value } => runtime.assign_expression(target, value, scope),
            Expr::Delete(operand) => runtime.delete(operand, scope),
            other => Err(Error::type_error(format!("{other} is not an operator"))),
        }
    }

    /// Finds operand roots by descending through operators.
    #[must_use]
    pub fn walk(&self) -> Vec<&'a Expr> {
        let mut roots = Vec::new();
        walk_operands(self.node, &mut roots);
        roots
    }
}

fn walk_operands<'a>(node: &'a Expr, accumulator: &mut Vec<&'a Expr>) {
    match node {
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            walk_operands(left, accumulator);
            walk_operands(right, accumulator);
        }
        Expr::Unary { operand, .. } | Expr::Delete(operand) => walk_operands(operand, accumulator),
        Expr::Conditional {
            condition,
            consequent,
            alternate,
        } => {
            walk_operands(condition, accumulator);
            walk_operands(consequent, accumulator);
            walk_operands(alternate, accumulator);
        }
        Expr::Assign { target, value } => {
            walk_operands(target, accumulator);
            walk_operands(value, accumulator);
        }
        other => accumulator.push(other),
    }
}

impl BinaryOp {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Power => "**",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::StrictEqual => "===",
            BinaryOp::StrictNotEqual => "!==",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Is => "is",
            BinaryOp::IsNot => "is not",
        }
    }
}

impl Runtime {
    pub(crate) fn evaluate_unary(
        &mut self,
        operator: UnaryOp,
        operand: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        match operator {
            UnaryOp::Positional => self.evaluate(operand, scope),
            // `$Person` is the class itself; `Person` is its instances.
            UnaryOp::TypeOf if matches!(operand, Expr::ClassRef(_)) => {
                Ok(Value::Class("Class".to_string()))
            }
            UnaryOp::TypeOf => Ok(type_of(&self.evaluate(operand, scope)?)),
            UnaryOp::Not => Ok(Value::Bool(!self.evaluate(operand, scope)?.truthy())),
            UnaryOp::Negate => Ok(Value::Number(-self.evaluate(operand, scope)?.to_number())),
            UnaryOp::Plus => Ok(Value::Number(self.evaluate(operand, scope)?.to_number())),
        }
    }

    pub(crate) fn evaluate_logical(
        &mut self,
        operator: LogicalOp,
        left: &Expr,
        right: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let left = match operator {
            LogicalOp::And | LogicalOp::Or => self.evaluate(left, scope)?,
            LogicalOp::Coalesce => self.evaluate_settled(left, scope)?,
        };
        let selected = match operator {
            LogicalOp::And => !left.truthy(),
            LogicalOp::Or => left.truthy(),
            LogicalOp::Coalesce => !left.is_null(),
        };

        if selected {
            Ok(left)
        } else {
            self.evaluate(right, scope)
        }
    }

    /// Evaluates the condition once and then only the selected branch, with
    /// the same truthiness and short-circuit reads as `&&` and `||`.
    pub(crate) fn evaluate_conditional(
        &mut self,
        condition: &Expr,
        consequent: &Expr,
        alternate: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let branch = if self.evaluate(condition, scope)?.truthy() {
            consequent
        } else {
            alternate
        };
        self.evaluate(branch, scope)
    }

    pub(crate) fn evaluate_binary(
        &mut self,
        operator: BinaryOp,
        left: &Expr,
        right: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let left = self.evaluate(left, scope)?;
        let right = match (operator, right) {
            (BinaryOp::Is | BinaryOp::IsNot, Expr::Identifier(word)) => self.read_word(word, scope),
            _ => self.evaluate(right, scope)?,
        };
        binary(operator, &left, &right)
    }
}

#[must_use]
pub fn type_of(value: &Value) -> Value {
    match value {
        Value::Object(_) => Value::Class("Object".to_string()),
        Value::Class(_) | Value::List(_) => Value::Class("List".to_string()),
        Value::Function(_) => Value::Class("Function".to_string()),
        Value::Number(_) => Value::String("number".to_string()),
        Value::String(_) => Value::String("string".to_string()),
        Value::Bool(_) => Value::String("boolean".to_string()),
        Value::Date(_) => Value::Class("Date".to_string()),
        Value::Regex(_) => Value::Class("RegExp".to_string()),
        Value::Null => Value::String("null".to_string()),
        Value::Undefined => Value::String("undefined".to_string()),
    }
}

pub fn binary(operator: BinaryOp, left: &Value, right: &Value) -> Result<Value> {
    use BinaryOp::{
        Add, Divide, Equal, Greater, GreaterEqual, Is, IsNot, Less, LessEqual, Modulo, Multiply,
        NotEqual, Power, StrictEqual, StrictNotEqual, Subtract,
    };

    // A read of something undefined makes the whole expression undefined, which
    // an assignment then stores as null.
    if matches!(
        operator,
        Add | Subtract | Multiply | Divide | Modulo | Power
    ) && (left.is_undefined() || right.is_undefined())
    {
        return Ok(Value::Undefined);
    }

    Ok(match operator {
        Add => match (left, right) {
            (Value::String(_), _) | (_, Value::String(_)) => {
                Value::String(format!("{left}{right}"))
            }
            _ => Value::Number(left.to_number() + right.to_number()),
        },
        Subtract => Value::Number(left.to_number() - right.to_number()),
        Multiply => Value::Number(left.to_number() * right.to_number()),
        Divide => Value::Number(left.to_number() / right.to_number()),
        Modulo => Value::Number(left.to_number() % right.to_number()),
        Power => Value::Number(left.to_number().powf(right.to_number())),

        Equal | Is => Value::Bool(loose_equal(left, right)),
        NotEqual | IsNot => Value::Bool(!loose_equal(left, right)),
        StrictEqual => Value::Bool(left == right),
        StrictNotEqual => Value::Bool(left != right),

        Less => Value::Bool(compare(left, right).is_some_and(Ordering::is_lt)),
        LessEqual => Value::Bool(compare(left, right).is_some_and(Ordering::is_le)),
        Greater => Value::Bool(compare(left, right).is_some_and(Ordering::is_gt)),
        GreaterEqual => Value::Bool(compare(left, right).is_some_and(Ordering::is_ge)),
    })
}

pub(crate) fn compare(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::String(left), Value::String(right)) => Some(left.as_str().cmp(right.as_str())),
        _ => left.to_number().partial_cmp(&right.to_number()),
    }
}

fn loose_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null | Value::Undefined, Value::Null | Value::Undefined) => true,
        (Value::Null | Value::Undefined, _) | (_, Value::Null | Value::Undefined) => false,
        (Value::String(left), Value::String(right)) | (Value::Class(left), Value::Class(right)) => {
            left == right
        }
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::List(left), Value::List(right)) => left == right,
        (Value::Bool(left), Value::Bool(right)) => left == right,
        _ => crate::number::equal(left.to_number(), right.to_number()),
    }
}
