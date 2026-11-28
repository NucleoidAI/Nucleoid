use super::{Parser, furthest};
use crate::error::Result;
use crate::lang::ast::lexer::tokenize_source;
use crate::lang::ast::parser::list::ListSeparators;

impl Parser {
    pub(super) fn from_source(source: &str) -> Result<Self> {
        let source = tokenize_source(source)?;
        Ok(Parser {
            line_breaks: source.line_breaks,
            ..Self::new(source.tokens)
        })
    }

    pub(super) fn attempt<T>(&mut self, parse: impl Fn(&mut Self) -> Result<T>) -> Result<T> {
        let line_breaks = self.line_breaks.clone();
        self.attempt_lines(&parse)
            .or_else(|accepted| self.attempt_ranges(&parse, accepted, line_breaks))
    }

    pub(super) fn attempt_lines<T>(
        &mut self,
        parse: &impl Fn(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let accepted = match self.attempt_original(parse) {
            Ok(parsed) => return Ok(parsed),
            Err(error) => error,
        };
        if self.line_breaks.is_empty() && !self.mentions_templates() {
            return Err(accepted);
        }
        *self = Parser {
            list_separators: ListSeparators::PhysicalLines,
            ..self.restarted()
        };
        self.attempt_original(parse)
            .map_err(|error| furthest(accepted, error))
    }

    fn attempt_original<T>(&mut self, parse: &impl Fn(&mut Self) -> Result<T>) -> Result<T> {
        self.attempt_selves(parse)
            .or_else(|accepted| self.attempt_sections(parse, accepted))
    }

    pub(super) fn restarted(&mut self) -> Self {
        Parser {
            line_breaks: std::mem::take(&mut self.line_breaks),
            list_separators: self.list_separators,
            ranges: self.ranges,
            ..Self::new(std::mem::take(&mut self.tokens))
        }
    }

    pub(super) fn rewound(&mut self) -> Self {
        Parser {
            selves: self.selves,
            sections: self.sections,
            ..self.restarted()
        }
    }
}
