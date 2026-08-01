//! The language: syntax, the tree it parses to, and how that tree is evaluated.
//!
//! Mirrors `ref/src/lang`. `ref/src/lang/$nuc` has no counterpart — it
//! serialises statements to and from a JSON datastore, whereas the graph here
//! holds typed [`ast::Stmt`] values directly.

pub mod ast;
pub mod estree;
pub mod evaluation;
