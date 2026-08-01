//! The concrete syntax layer: source text in, [`ast`](crate::lang::ast) out,
//! and back again.
//!
//! `ref/src/lang/estree` wraps `acorn` for parsing and `escodegen` for
//! generation. The Rust crate owns both ends, so [`lexer`] has no counterpart
//! there while [`parser`] and [`generator`] do.

pub mod generator;
pub mod lexer;
pub mod parser;

pub use parser::{Program, parse, parse_expression, parse_program};
