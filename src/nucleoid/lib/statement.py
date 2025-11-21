"""
Statement compilation utility using Python's AST parser.
Provides a simple interface for parsing Python code strings into AST nodes.
"""

import ast
from typing import Any, List


class Statement:
    """Statement compiler for parsing code strings."""

    def compile(self, source: str) -> List[Any]:
        """
        Parse source code into AST nodes.

        Args:
            source: Source code as a string

        Returns:
            List of AST statement nodes

        Raises:
            SyntaxError: If the source code is invalid
        """
        try:
            tree = ast.parse(source)
            return tree.body
        except SyntaxError as e:
            raise SyntaxError(f"Failed to parse source code: {e}") from e


# Create singleton instance
statement_instance = Statement()


def compile(source: str) -> List[ast.stmt]:
    """
    Parse source code into AST nodes.

    Args:
        source: Source code as a string

    Returns:
        List of AST statement nodes

    Raises:
        SyntaxError: If the source code is invalid Python
    """
    return statement_instance.compile(source)
