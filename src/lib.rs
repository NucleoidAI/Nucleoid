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
//! | [`graph`] | Dependency nodes, value edges and structural observations |
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

#![warn(
    clippy::pedantic,
    clippy::debug_assert_with_mut_call,
    clippy::exit,
    clippy::expect_used,
    clippy::format_push_string,
    clippy::indexing_slicing,
    clippy::infinite_loop,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::shadow_unrelated,
    clippy::string_slice,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unused_result_ok,
    clippy::unwrap_used,
    missing_debug_implementations,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::shadow_unrelated,
        clippy::string_slice,
        clippy::too_many_lines,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

pub mod builtins;
pub mod error;
pub mod expression;
pub mod graph;
mod history;
pub mod lang;
pub mod nuc;
mod number;
pub mod reasoning;
pub mod runtime;
pub mod scope;
pub mod stack;
pub mod state;
pub mod statement;
pub mod transaction;
pub mod value;

pub use error::{Error, ErrorKind, Position, Result};
pub use lang::ast::lexer::SyntaxOptions;
pub use runtime::{AssertionFailure, Runtime};
pub use value::{ObjectId, Value};
