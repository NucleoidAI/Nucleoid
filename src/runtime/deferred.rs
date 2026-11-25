use crate::error::{Error, Position, Result};
use crate::lang::ast::Stmt;
use crate::lang::evaluation::Flow;
use crate::nuc::expression::is_assertion;
use crate::runtime::{MAX_DEPTH, Runtime};
use crate::scope::Scope;
use crate::value::Value;

struct Suite<'a> {
    body: &'a [Stmt],
    position: Option<Position>,
}

fn positioned(result: Result<Flow>, position: Option<Position>) -> Result<Flow> {
    result.map_err(|error| match position {
        Some(position) => error.or_at(position),
        None => error,
    })
}

impl Runtime {
    pub(super) fn execute_statements(
        &mut self,
        statements: &[Stmt],
        positions: &[Position],
        scope: &mut Scope,
    ) -> Result<Flow> {
        let body_savepoint = statements
            .iter()
            .any(|statement| matches!(statement, Stmt::Defer(_)))
            .then(|| self.savepoint());
        let mut suites = Vec::new();
        let mut pending = self.sequence(statements, positions, scope, &mut suites);

        if !suites.is_empty() && pending.is_err() {
            if let Some(mark) = body_savepoint {
                self.rollback_to(mark);
            }
        }

        for suite in suites.into_iter().rev() {
            let cleanup_savepoint = self.savepoint();
            let result = positioned(self.deferred_suite(suite.body, scope), suite.position);
            if result.is_err() {
                self.rollback_to(cleanup_savepoint);
            }
            if !matches!(result, Ok(Flow::Normal(_))) {
                pending = result;
            }
        }
        pending
    }

    fn sequence<'a>(
        &mut self,
        statements: &'a [Stmt],
        positions: &[Position],
        scope: &mut Scope,
        suites: &mut Vec<Suite<'a>>,
    ) -> Result<Flow> {
        let mut last = Value::Null;
        for (index, statement) in statements.iter().enumerate() {
            if !positions.is_empty() {
                debug_assert_eq!(self.transaction.hypotheses(), Vec::<usize>::new());
            }
            let position = positions.get(index).copied();
            if let Stmt::Defer(body) = statement {
                suites.push(Suite { body, position });
                continue;
            }
            match positioned(self.execute(statement, scope), position)? {
                returned @ Flow::Return(_) => return Ok(returned),
                Flow::Normal(value) => {
                    if !is_assertion(statement) {
                        last = value;
                    }
                }
            }
        }
        Ok(Flow::Normal(last))
    }

    fn deferred_suite(&mut self, statements: &[Stmt], scope: &mut Scope) -> Result<Flow> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum statement depth exceeded"));
        }
        let depth = self.depth;
        let result = self.execute_all(statements, scope);
        debug_assert_eq!(self.depth, depth);
        self.depth -= 1;
        result
    }
}
