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
//! The modules follow `ref/`, the archived JavaScript implementation, so that
//! the two can be read side by side:
//!
//! | this crate | `ref/` |
//! | --- | --- |
//! | [`lang::estree`] | `src/lang/estree` |
//! | [`lang::ast`] | `src/lang/ast` |
//! | [`lang::evaluation`] | `src/lang/Evaluation.js` |
//! | [`nuc`] | `src/nuc` |
//! | [`graph`] | `src/graph.js` |
//! | [`state`] | `src/state.js` |
//! | [`scope`] | `src/Scope.js` |
//! | [`stack`] | `src/stack.js` |
//! | [`statement`] | `src/statement.js` |
//! | [`transaction`] | `src/transaction.js` |
//! | [`runtime`] | `src/runtime.js` |
//!
//! `ref`'s server infrastructure — `express.js`, `routes/`, `datastore.js`,
//! `cluster.js`, `cache.js`, `config.js`, `event.js`, `process.js` — has no
//! counterpart: this crate is the language, not a service around it. Nor does
//! `src/lang/$nuc`, which serialises statements for that datastore.

pub mod builtins;
pub mod error;
pub mod graph;
pub mod lang;
pub mod nuc;
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
