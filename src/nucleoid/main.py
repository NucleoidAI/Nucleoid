"""Parser entrypoint for Nucleoid, a logic language for LLMs."""

from __future__ import annotations

import ast


def run(source: str) -> ast.AST:
    """Parse source code into a native Python AST."""
    if not isinstance(source, str):
        raise TypeError("source must be a string")

    return ast.parse(source)
