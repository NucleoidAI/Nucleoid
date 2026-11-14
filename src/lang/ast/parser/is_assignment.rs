use std::collections::HashMap;

use super::expression::PathTarget;
use super::{ArgumentContext, ExpressionContext, Parser, PythonCondition};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, ObjectEntry, Stmt, UnaryOp};

fn begins_expression(token: &Token) -> bool {
    matches!(
        token,
        Token::Identifier(_)
            | Token::ClassRef(_)
            | Token::Number(_)
            | Token::Str(_)
            | Token::Template(_)
            | Token::Regex(_)
            | Token::LeftParen
            | Token::LeftBracket
            | Token::LeftBrace
            | Token::Bang
            | Token::Minus
            | Token::Plus
            | Token::Keyword(
                Keyword::True
                    | Keyword::False
                    | Keyword::Null
                    | Keyword::This
                    | Keyword::Super
                    | Keyword::New
                    | Keyword::Delete
                    | Keyword::Function
                    | Keyword::Not
                    | Keyword::TypeOf
            )
    )
}

fn word(value: Expr) -> Expr {
    Expr::Unary {
        operator: UnaryOp::Word,
        operand: Box::new(value),
    }
}

fn is_word(start: usize, end: usize, value: &Expr) -> bool {
    end == start + 1 && matches!(value, Expr::Identifier(_))
}

fn without_words(mut target: Expr) -> Expr {
    let mut pending = vec![&mut target];
    while let Some(expression) = pending.pop() {
        if let Expr::Unary {
            operator: UnaryOp::Word,
            operand,
        } = expression
        {
            let name = std::mem::replace(operand.as_mut(), Expr::Null);
            *expression = name;
        } else {
            push_children(expression, &mut pending);
        }
    }
    target
}

fn push_children<'a>(expression: &'a mut Expr, pending: &mut Vec<&'a mut Expr>) {
    match expression {
        Expr::Member { object, .. } | Expr::Delete(object) => pending.push(object),
        Expr::Version { reference, .. } => pending.push(reference),
        Expr::Reason { source, .. } => pending.push(source),
        Expr::Of { operand, .. } | Expr::Unary { operand, .. } => pending.push(operand),
        Expr::Index { object, index } => pending.extend([object, index].map(Box::as_mut)),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            pending.extend([left, right].map(Box::as_mut));
        }
        Expr::Assign { target, value } => pending.extend([target, value].map(Box::as_mut)),
        Expr::Depth { source, limit } => pending.extend([source, limit].map(Box::as_mut)),
        Expr::Path { source, target } => pending.extend([source, target].map(Box::as_mut)),
        Expr::Combine { source, other, .. } => pending.extend([source, other].map(Box::as_mut)),
        Expr::Quantified {
            collection,
            condition,
            ..
        } => pending.extend([collection, condition].map(Box::as_mut)),
        Expr::Slice { object, start, end } => {
            pending.push(object);
            pending.extend(start.iter_mut().chain(end).map(Box::as_mut));
        }
        Expr::Conditional {
            condition,
            consequent,
            alternate,
        } => pending.extend([condition, consequent, alternate].map(Box::as_mut)),
        Expr::Call { callee, arguments } => {
            pending.push(callee);
            pending.extend(arguments);
        }
        Expr::List(items) | Expr::Super(items) => pending.extend(items),
        Expr::ObjectLiteral(entries) => pending.extend(entries.iter_mut().map(|(_, value)| value)),
        Expr::SpreadObject(entries) => {
            pending.extend(entries.iter_mut().map(|entry| match entry {
                ObjectEntry::Property(_, value) | ObjectEntry::Spread(value) => value,
            }));
        }
        Expr::Null
        | Expr::Bool(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::Template(_)
        | Expr::Regex(_)
        | Expr::Identifier(_)
        | Expr::ClassRef(_)
        | Expr::ObjectRef(_)
        | Expr::This
        | Expr::Function(_)
        | Expr::Model
        | Expr::Enum(_) => {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Branches {
    Plain,
    Words,
}

impl Branches {
    pub(super) fn mark(self, start: usize, end: usize, branch: Expr) -> Expr {
        if self == Branches::Words && is_word(start, end, &branch) {
            word(branch)
        } else {
            branch
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Items {
    Plain,
    Variables,
    Words,
}

impl Items {
    pub(super) fn mark(self, start: usize, end: usize, item: &mut Expr) {
        if self == Items::Words && is_word(start, end, item) {
            let name = std::mem::replace(item, Expr::Null);
            *item = word(name);
        }
    }
}

impl Parser {
    pub(super) fn starts_is_assignment(&self, target: &Expr) -> bool {
        matches!(self.peek(), Token::Identifier(name) if name == "is")
            && (begins_expression(self.peek_at(1)) || self.starts_at_variable(1))
            && PathTarget::of(target) == PathTarget::Plain
    }

    pub(super) fn is_assignment(&mut self, target: Expr) -> Result<Stmt> {
        self.advance();
        let start = self.position;
        self.record_value_start();
        let items = std::mem::replace(&mut self.items, Items::Words);
        let value = self.without_reads(|parser| {
            parser.nested_expression(|parser| {
                parser.nested_expression_in(ExpressionContext::Ordinary, |value| {
                    value.assignment_expression(Branches::Words)
                })
            })
        });
        self.items = items;
        let value = value?;
        let bare = is_word(start, self.position, &value);
        let statement = self.expression_tail(Expr::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })?;
        Ok(match statement {
            Stmt::Assign {
                target: path,
                value: name,
            } if bare => Stmt::Assign {
                target: path,
                value: word(name),
            },
            statement => statement,
        })
    }

    pub(super) fn item(&mut self) -> Result<Expr> {
        self.nested_expression_in(ExpressionContext::Ordinary, Self::item_expression)
    }

    pub(super) fn object_item(&mut self) -> Result<Expr> {
        self.nested_expression_in(ExpressionContext::ObjectValue, Self::item_expression)
    }

    fn item_expression(&mut self) -> Result<Expr> {
        let branches = match self.items {
            Items::Plain | Items::Variables => Branches::Plain,
            Items::Words => Branches::Words,
        };
        self.assignment_expression(branches)
    }

    pub(super) fn starts_item_label(&self, context: ArgumentContext) -> bool {
        self.items == Items::Words
            && matches!(context, ArgumentContext::PotentialConstructor)
            && matches!(self.peek(), Token::Identifier(_))
            && self.peek_at(1) == &Token::Assign
    }

    pub(super) fn item_label(&mut self) -> Result<Expr> {
        self.nested_expression_in(ExpressionContext::Ordinary, |label| {
            let target = label.primary()?;
            debug_assert!(matches!(target, Expr::Identifier(_)) && label.check(&Token::Assign));
            label.advance();
            let start = label.position;
            let mut value = label.item()?;
            label.items.mark(start, label.position, &mut value);
            Ok(Expr::Assign {
                target: Box::new(target),
                value: Box::new(value),
            })
        })
    }

    pub(super) fn leave_items(&mut self) -> Items {
        std::mem::replace(&mut self.items, Items::Plain)
    }

    pub(super) fn conditions(&mut self) -> &mut HashMap<(usize, usize), Option<PythonCondition>> {
        match self.items {
            Items::Plain => &mut self.python_conditions,
            Items::Variables => &mut self.variable_conditions,
            Items::Words => &mut self.item_conditions,
        }
    }

    pub(super) fn unmark(&self, branches: Branches, target: Expr) -> Expr {
        if branches == Branches::Plain && self.items != Items::Words {
            target
        } else {
            without_words(target)
        }
    }
}
