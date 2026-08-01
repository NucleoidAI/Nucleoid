//! Source text to statements. Mirrors `ref/src/statement.js`, which is likewise
//! the one place the rest of the runtime reaches the parser through.

use crate::error::Result;
use crate::lang::estree::parser::{self, Program};

/// Compiles a program, reporting the first syntax error.
pub fn compile(source: &str) -> Result<Program> {
    parser::parse_program(source)
}
