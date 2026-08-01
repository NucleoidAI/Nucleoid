//! Unary, binary and logical operators. Mirrors `ref/src/lang/ast/Operator.js`.

use crate::error::Result;
use crate::lang::ast::{BinaryOp, Expr, LogicalOp, UnaryOp};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl BinaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::StrictEqual => "===",
            BinaryOp::StrictNotEqual => "!==",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
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
        if operator == UnaryOp::TypeOf {
            // `$Person` is the class itself; `Person` is its instances.
            if let Expr::ClassRef(_) = operand {
                return Ok(Value::Class("Class".to_string()));
            }

            let value = self.evaluate(operand, scope)?;
            return Ok(type_of(&value));
        }

        let value = self.evaluate(operand, scope)?;

        Ok(match operator {
            UnaryOp::Not => Value::Bool(!value.truthy()),
            UnaryOp::Negate => Value::Number(-value.to_number()),
            UnaryOp::Plus => Value::Number(value.to_number()),
            UnaryOp::TypeOf => unreachable!("handled above"),
        })
    }

    pub(crate) fn evaluate_logical(
        &mut self,
        operator: LogicalOp,
        left: &Expr,
        right: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let left = self.evaluate(left, scope)?;

        match operator {
            LogicalOp::And => {
                if left.truthy() {
                    self.evaluate(right, scope)
                } else {
                    Ok(left)
                }
            }
            LogicalOp::Or => {
                if left.truthy() {
                    Ok(left)
                } else {
                    self.evaluate(right, scope)
                }
            }
        }
    }

    pub(crate) fn evaluate_binary(
        &mut self,
        operator: BinaryOp,
        left: &Expr,
        right: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let left = self.evaluate(left, scope)?;
        let right = self.evaluate(right, scope)?;
        binary(operator, &left, &right)
    }
}

pub fn type_of(value: &Value) -> Value {
    match value {
        Value::Object(_) => Value::Class("Object".to_string()),
        Value::Class(_) => Value::Class("List".to_string()),
        Value::List(_) => Value::Class("List".to_string()),
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
    use BinaryOp::*;

    // A read of something undefined makes the whole expression undefined, which
    // an assignment then stores as null.
    if matches!(operator, Add | Subtract | Multiply | Divide | Modulo)
        && (left.is_undefined() || right.is_undefined())
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

        Equal => Value::Bool(loose_equal(left, right)),
        NotEqual => Value::Bool(!loose_equal(left, right)),
        StrictEqual => Value::Bool(left == right),
        StrictNotEqual => Value::Bool(left != right),

        Less | LessEqual | Greater | GreaterEqual => {
            let ordering = match (left, right) {
                (Value::String(left), Value::String(right)) => {
                    Some(left.as_str().cmp(right.as_str()))
                }
                _ => left.to_number().partial_cmp(&right.to_number()),
            };

            let Some(ordering) = ordering else {
                return Ok(Value::Bool(false));
            };

            Value::Bool(match operator {
                Less => ordering.is_lt(),
                LessEqual => ordering.is_le(),
                Greater => ordering.is_gt(),
                GreaterEqual => ordering.is_ge(),
                _ => unreachable!("only comparisons reach here"),
            })
        }
    })
}

fn loose_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null | Value::Undefined, Value::Null | Value::Undefined) => true,
        (Value::Null | Value::Undefined, _) | (_, Value::Null | Value::Undefined) => false,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::Class(left), Value::Class(right)) => left == right,
        (Value::List(left), Value::List(right)) => left == right,
        (Value::Bool(left), Value::Bool(right)) => left == right,
        _ => left.to_number() == right.to_number(),
    }
}
