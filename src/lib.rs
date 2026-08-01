//! Nucleoid is a declarative logic runtime.
//!
//! Statements are filed in a dependency graph as they run, so an assignment is
//! a standing rule rather than a one-off: when something it reads changes, it is
//! re-evaluated.
//!
//! ```
//! use nucleoid::Runtime;
//!
//! let mut runtime = Runtime::new();
//! runtime.run("a = 1").unwrap();
//! runtime.run("b = a + 1").unwrap();
//! runtime.run("a = 2").unwrap();
//!
//! assert_eq!(runtime.run("b").unwrap().to_string(), "3");
//! ```

pub mod ast;
pub mod builtins;
pub mod error;
mod eval;
pub mod graph;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod scope;
pub mod state;
pub mod transaction;
pub mod value;

pub use error::{Error, ErrorKind, Result};
pub use runtime::{AssertionFailure, Runtime};
pub use value::{ObjectId, Value};
