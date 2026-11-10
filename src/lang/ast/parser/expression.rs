use super::is_assignment::Branches;
use super::{CompoundOp, ExpressionContext, MAX_CHAIN, MAX_NESTING, Parser, PythonCondition};
use crate::error::{Error, Result};
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{BinaryOp, Expr, LogicalOp};
use crate::reasoning::Stage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PathTarget {
    Plain,
    TooManyParts,
    Invalid,
}

impl PathTarget {
    pub(super) fn of(mut target: &Expr) -> Self {
        let mut parts = 0;
        while let Expr::Member { object, .. }
        | Expr::Version {
            reference: object, ..
        } = target
        {
            parts += 1;
            if parts > MAX_CHAIN {
                return PathTarget::TooManyParts;
            }
            target = object;
        }

        match target {
            Expr::Identifier(_) => PathTarget::Plain,
            Expr::ClassRef(_) | Expr::This if parts > 0 => PathTarget::Plain,
            _ => PathTarget::Invalid,
        }
    }
}

impl Parser {
    pub(super) fn expression(&mut self) -> Result<Expr> {
        self.expression_in(ExpressionContext::Ordinary)
    }

    /// An `if`/`else`/`for` header, subscript start or conditional consequent:
    /// its top-level `:` belongs to the enclosing construct, never to a
    /// contextual `lambda`.
    pub(super) fn clause_expression(&mut self) -> Result<Expr> {
        self.clause_expression_with(Branches::Plain)
    }

    fn clause_expression_with(&mut self, branches: Branches) -> Result<Expr> {
        self.within(true, |parser| {
            parser.nested_expression_in(ExpressionContext::Clause, |clause| {
                clause.assignment_expression(branches)
            })
        })
    }

    pub(super) fn expression_in(&mut self, context: ExpressionContext) -> Result<Expr> {
        self.nested_expression_in(context, Self::expression_inner)
    }

    /// Runs one nested `parse` in `context`, then restores the enclosing
    /// context whether or not it succeeded.
    pub(super) fn nested_expression_in(
        &mut self,
        context: ExpressionContext,
        parse: impl FnOnce(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        let outer = std::mem::replace(&mut self.context, context);
        let expression = self.nested_expression(parse);
        self.context = outer;
        expression
    }

    pub(super) fn nested_expression(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Expressions are nested too deeply"));
        }

        let depth = self.depth;
        let result = parse(self);
        debug_assert_eq!(self.depth, depth);
        self.depth -= 1;
        result
    }

    fn expression_inner(&mut self) -> Result<Expr> {
        self.assignment_expression(Branches::Plain)
    }

    pub(super) fn assignment_expression(&mut self, branches: Branches) -> Result<Expr> {
        let start = self.position;
        let left = self.conditional_with(branches)?;
        self.assignment(start, branches, left)
    }

    fn assignment(&mut self, start: usize, branches: Branches, left: Expr) -> Result<Expr> {
        let operator = match self.peek() {
            Token::Assign => None,
            Token::PlusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Add)),
            Token::MinusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Subtract)),
            Token::StarAssign => Some(CompoundOp::Arithmetic(BinaryOp::Multiply)),
            Token::StarStarAssign => Some(CompoundOp::Arithmetic(BinaryOp::Power)),
            Token::SlashAssign => Some(CompoundOp::Arithmetic(BinaryOp::Divide)),
            Token::PercentAssign => Some(CompoundOp::Arithmetic(BinaryOp::Modulo)),
            Token::AndAndAssign => Some(CompoundOp::Logical(LogicalOp::And)),
            Token::OrOrAssign => Some(CompoundOp::Logical(LogicalOp::Or)),
            _ => return Ok(left),
        };
        self.check_target_start(start)?;
        if let Some(variable) = self.at_variable_since(start) {
            return self.reject_at_variable(variable);
        }
        let left = self.unmark(branches, left);

        if operator.is_some() {
            self.check_compound_target(&left)?;
        }

        self.advance();
        let items = self.leave_items();
        let value = self.receiving(&left, |parser| parser.expression_in(parser.context));
        self.items = items;
        let value = value?;
        let value = match operator {
            Some(operator) => {
                let left = Box::new(left.clone());
                let right = Box::new(value);
                match operator {
                    CompoundOp::Arithmetic(operator) => Expr::Binary {
                        operator,
                        left,
                        right,
                    },
                    CompoundOp::Logical(operator) => Expr::Logical {
                        operator,
                        left,
                        right,
                    },
                }
            }
            None => value,
        };
        Ok(Expr::Assign {
            target: Box::new(left),
            value: Box::new(value),
        })
    }

    /// Only clone bounded, plain paths. A computed target containing another
    /// compound assignment could otherwise double its tree at each lowering.
    fn check_compound_target(&self, target: &Expr) -> Result<()> {
        self.check_path_target(target, "Invalid compound assignment target")
    }

    pub(super) fn check_path_target(&self, target: &Expr, invalid: &str) -> Result<()> {
        match PathTarget::of(target) {
            PathTarget::Plain => Ok(()),
            PathTarget::TooManyParts => Err(self.error("Expression has too many parts")),
            PathTarget::Invalid => Err(self.error(invalid)),
        }
    }

    /// `condition ? consequent : alternate` or Python's
    /// `consequent if condition else alternate`: one right-associative level
    /// above assignment and below the reasoning pipeline. Both spellings build
    /// the same node. The consequent after `?` is a full expression whose
    /// following `:` belongs to the conditional; the alternate continues in
    /// the enclosing context.
    pub(super) fn conditional(&mut self) -> Result<Expr> {
        self.conditional_with(Branches::Plain)
    }

    fn conditional_with(&mut self, branches: Branches) -> Result<Expr> {
        let start = self.position;
        let operand = self.pipeline()?;
        let operand = self.coalescing(operand)?;
        let end = self.position;

        let (condition, consequent) = if self.eat(&Token::Question) {
            let consequent =
                self.branch(branches, |parser| parser.clause_expression_with(branches))?;
            self.expect(&Token::Colon)?;
            (operand, consequent)
        } else if let Some(condition) = self.python_condition() {
            (condition, branches.mark(start, end, operand))
        } else {
            return Ok(operand);
        };

        let alternate = self.branch(branches, |parser| {
            parser.nested_expression(|alternate| alternate.conditional_with(branches))
        })?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            consequent: Box::new(consequent),
            alternate: Box::new(alternate),
        })
    }

    fn branch(
        &mut self,
        branches: Branches,
        parse: impl FnOnce(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        let start = self.position;
        let branch = parse(self)?;
        Ok(branches.mark(start, self.position, branch))
    }

    /// Qualifies a Python-style conditional after a complete operand: `if`,
    /// one condition,
    /// and an immediately following `else`, which is consumed. Any other
    /// outcome consumes nothing, so the statement parser still reads `if` as
    /// an adjacent statement with its original positions and diagnostics.
    /// Template interpolations keep ignoring a trailing `if`.
    pub(super) fn python_condition(&mut self) -> Option<Expr> {
        if matches!(
            self.context,
            ExpressionContext::Interpolation | ExpressionContext::ComprehensionSource
        ) || self.peek() != &Token::Keyword(Keyword::If)
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.conditions().get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_python_condition();
            self.conditions().insert(attempt, outcome.clone());
            outcome
        };
        let PythonCondition {
            condition,
            end,
            misplaced_every,
        } = outcome?;
        self.position = end;
        self.adopt(misplaced_every);
        Some(condition)
    }

    /// Parses one Python-style condition after `if` without keeping any
    /// position, depth or context change.
    fn attempt_python_condition(&mut self) -> Option<PythonCondition> {
        let start = self.position;
        self.advance();
        let (condition, misplaced_every) = self.tentatively(|parser| {
            parser.within(true, |parser| {
                parser.nested_expression_in(ExpressionContext::Clause, Self::coalescing_expression)
            })
        });
        let outcome = match condition {
            Ok(condition) if self.eat_keyword(Keyword::Else) => Some(PythonCondition {
                condition,
                end: self.position,
                misplaced_every,
            }),
            _ => None,
        };
        self.position = start;
        outcome
    }

    fn coalescing_expression(&mut self) -> Result<Expr> {
        let operand = self.pipeline()?;
        self.coalescing(operand)
    }

    fn coalescing(&mut self, first: Expr) -> Result<Expr> {
        let mut lefts = Vec::new();
        let mut last = first;

        while self.eat(&Token::QuestionQuestion) {
            if lefts.len() >= MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let next = self.pipeline()?;
            lefts.push(std::mem::replace(&mut last, next));
        }

        Ok(lefts
            .into_iter()
            .rev()
            .fold(last, |right, left| Expr::Logical {
                operator: LogicalOp::Coalesce,
                left: Box::new(left),
                right: Box::new(right),
            }))
    }

    fn pipeline(&mut self) -> Result<Expr> {
        let mut left = self.logical_or()?;

        let mut chain = 0;

        while self.check(&Token::Pipe) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            if let Expr::Identifier(name) = &left {
                if name == "model" {
                    left = Expr::Model;
                }
            }

            left = self.stage(left)?;
        }

        Ok(left)
    }

    fn stage(&mut self, source: Expr) -> Result<Expr> {
        if let Token::Identifier(name) = self.peek() {
            let stage = match name.as_str() {
                "diff" => Some(Stage::Diff),
                other => Stage::from_name(other),
            };
            if let Some(stage) = stage {
                self.advance();
                return Ok(Expr::Reason {
                    stage,
                    source: Box::new(source),
                });
            }

            if name == "depth" && self.peek_at(1) == &Token::LeftParen {
                return Ok(Expr::Depth {
                    source: Box::new(source),
                    limit: self.stage_argument()?,
                });
            }

            if name == "path" && self.peek_at(1) == &Token::LeftParen {
                return Ok(Expr::Path {
                    source: Box::new(source),
                    target: self.stage_argument()?,
                });
            }

            return Err(Error::type_error(format!("{name} is not an operation")));
        }

        Err(self.error("Expected an operation"))
    }

    fn stage_argument(&mut self) -> Result<Box<Expr>> {
        self.position += 2;
        let argument = self.within(true, Self::expression)?;
        self.expect(&Token::RightParen)?;
        Ok(Box::new(argument))
    }
}
