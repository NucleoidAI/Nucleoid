use crate::value::Value;
use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    #[error("SyntaxError: {0}")]
    Syntax(String),

    #[error("ReferenceError: {0}")]
    Reference(String),

    #[error("TypeError: {0}")]
    Type(String),

    #[error("{0}")]
    Thrown(Thrown),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Thrown(pub Value);

impl fmt::Display for Thrown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error {
    pub fn reference(message: impl Into<String>) -> Self {
        Error::Reference(message.into())
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Error::Type(message.into())
    }

    pub fn syntax(message: impl Into<String>) -> Self {
        Error::Syntax(message.into())
    }

    pub fn not_defined(name: impl fmt::Display) -> Self {
        Error::Reference(format!("{name} is not defined"))
    }

    pub fn thrown(value: Value) -> Self {
        Error::Thrown(Thrown(value))
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Syntax(_) => ErrorKind::Syntax,
            Error::Reference(_) => ErrorKind::Reference,
            Error::Type(_) => ErrorKind::Type,
            Error::Thrown(_) => ErrorKind::Thrown,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Error::Syntax(message) | Error::Reference(message) | Error::Type(message) => {
                message.clone()
            }
            Error::Thrown(thrown) => thrown.0.to_string(),
        }
    }
}

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
