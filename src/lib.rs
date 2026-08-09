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
//! The modules follow `ref/`, the archived JavaScript implementation, file for
//! file and class for class, so that the two can be read side by side:
//!
//! | this crate | `ref/` |
//! | --- | --- |
//! | [`lang::estree`] | `src/lang/estree` |
//! | [`lang::ast`], [`lang::ast::Ast`] | `src/lang/ast`, `Node.js` |
//! | [`lang::evaluation`] | `src/lang/Evaluation.js` |
//! | [`nuc`], [`nuc::Nuc`] | `src/nuc`, `NODE.js` |
//! | [`expression`] | `src/Expression.js` |
//! | [`graph`] | `src/graph.js` |
//! | [`state`] | `src/state.js` |
//! | [`scope`] | `src/Scope.js` |
//! | [`stack`] | `src/stack.js` |
//! | [`statement`] | `src/statement.js` |
//! | [`transaction`] | `src/transaction.js` |
//! | [`runtime`] | `src/runtime.js` |
//!
//! Three modules answer to nothing in `ref`, because they are what being typed
//! costs: [`value`] is the runtime value as a closed enum where `ref` has
//! whatever JavaScript handed it, [`error`] is the failure as a typed enum
//! returned through [`Result`] where `ref` throws, and [`builtins`] gathers the
//! standard objects that `ref` reaches by leaving them to its host.
//!
//! `ref` is untyped JavaScript, so its node kinds are classes reached by
//! dynamic dispatch and its `$CLASS`/`$INSTANCE` variants are subclasses. The
//! kinds are a closed set, so here they are the [`nuc::Nuc`] and
//! [`lang::ast::Ast`] enums with one type per class behind them, and the
//! context a node runs in is a field rather than a subclass. Statements carry
//! `ref`'s four phases — `before`, `run`, `graph`, `after` — in the order
//! `ref/src/stack.js` calls them.
//!
//! `ref`'s server infrastructure — `express.js`, `routes/`, `datastore.js`,
//! `cluster.js`, `cache.js`, `config.js`, `event.js`, `process.js` — has no
//! counterpart: this crate is the language, not a service around it. Nor does
//! `src/lang/$nuc`, which serialises statements for that datastore.

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
