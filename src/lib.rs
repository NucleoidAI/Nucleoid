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
//!
//! # Layout
//!
//! | module | responsibility |
//! | --- | --- |
//! | [`lang::ast::parser`], [`lang::ast::generator`] | Parse and render source |
//! | [`lang::ast`], [`lang::ast::Ast`] | Typed syntax and expression evaluators |
//! | [`lang::evaluation`] | Scoped evaluation and dependency tracking |
//! | [`nuc`], [`nuc::Nuc`] | Executable statement kinds and their lifecycle |
//! | [`expression`] | Expression-tree queries |
//! | [`graph`] | Dependency nodes and edges |
//! | [`state`] | Variables, objects, classes and functions |
//! | [`scope`] | Local bindings and execution context |
//! | [`stack`] | Dependency propagation queue |
//! | [`statement`] | Source compilation |
//! | [`transaction`] | Undo logs, savepoints and rollback |
//! | [`runtime`] | Statement execution and the public runtime API |
//! | [`value`], [`error`] | Runtime values and typed errors through [`Result`] |
//! | [`builtins`] | Standard objects and operations |
//!
//! [`nuc::Nuc`] and [`lang::ast::Ast`] are closed enums with dedicated types for
//! their variants. Execution context is carried by scopes and graph nodes.
//! Statements follow four phases: `before`, `run`, `graph`, and `after`.

pub mod builtins;
pub mod error;
pub mod expression;
pub mod graph;
pub mod lang;
pub mod nuc;
pub mod reasoning;
pub mod runtime;
pub mod scope;
pub mod stack;
pub mod state;
pub mod statement;
pub mod transaction;
pub mod value;

pub use error::{Error, ErrorKind, Position, Result};
pub use runtime::{AssertionFailure, Runtime};
pub use value::{ObjectId, Value};
