use crate::value::Value;
use std::fmt;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

/// A place in the source. Lines and columns are 1-based, and columns count
/// characters rather than bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// What went wrong, without regard to where. The rendering of each variant is
/// part of the language's observable behaviour: it is the value a `catch` sees.
#[derive(Debug, Clone, PartialEq, ThisError)]
enum Detail {
    #[error("SyntaxError: {0}")]
    Syntax(String),

    #[error("ReferenceError: {0}")]
    Reference(String),

    #[error("TypeError: {0}")]
    Type(String),

    #[error("{0}")]
    Thrown(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    detail: Detail,
    position: Option<Position>,
}

impl Error {
    fn new(detail: Detail) -> Self {
        Error {
            detail,
            position: None,
        }
    }

    pub fn syntax(message: impl Into<String>) -> Self {
        Error::new(Detail::Syntax(message.into()))
    }

    pub fn reference(message: impl Into<String>) -> Self {
        Error::new(Detail::Reference(message.into()))
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Error::new(Detail::Type(message.into()))
    }

    pub fn not_defined(name: impl fmt::Display) -> Self {
        Error::reference(format!("{name} is not defined"))
    }

    pub fn thrown(value: Value) -> Self {
        Error::new(Detail::Thrown(value))
    }

    /// Places the error in the source, replacing any position it already had.
    pub fn at(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Places the error only if it is not already placed, so the innermost
    /// report wins.
    pub fn or_at(mut self, position: Position) -> Self {
        self.position = self.position.or(Some(position));
        self
    }

    pub fn position(&self) -> Option<Position> {
        self.position
    }

    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::Syntax(_) => ErrorKind::Syntax,
            Detail::Reference(_) => ErrorKind::Reference,
            Detail::Type(_) => ErrorKind::Type,
            Detail::Thrown(_) => ErrorKind::Thrown,
        }
    }

    pub fn message(&self) -> String {
        match &self.detail {
            Detail::Syntax(message) | Detail::Reference(message) | Detail::Type(message) => {
                message.clone()
            }
            Detail::Thrown(value) => value.to_string(),
        }
    }

    /// The value a `throw` raised, when that is what this is.
    pub fn thrown_value(&self) -> Option<&Value> {
        match &self.detail {
            Detail::Thrown(value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.detail)?;

        if let Some(position) = self.position {
            write!(f, " ({position})")?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Reference,
    Type,
    Thrown,
}

impl ErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Syntax => "SyntaxError",
            ErrorKind::Reference => "ReferenceError",
            ErrorKind::Type => "TypeError",
            ErrorKind::Thrown => "Thrown",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
