"""
Statement compilation utility using Python's AST parser.
Provides a simple interface for parsing Python code strings into AST nodes.
"""

import ast
from typing import Any


def compile(source: str) -> list[ast.stmt]:
    """
    Parse Python source code into AST nodes.

    Args:
        source: Python source code as a string

    Returns:
        List of AST statement nodes

    Raises:
        SyntaxError: If the source code is invalid Python
    """
    try:
        tree = ast.parse(source)
        return tree.body
    except SyntaxError as e:
        raise SyntaxError(f"Failed to parse source code: {e}") from e
