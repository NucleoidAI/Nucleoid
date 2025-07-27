"""
Statement compilation and parsing for Nucleoid.

This module handles the compilation of Nucleoid statement strings into
executable AST nodes, corresponding to the TypeScript statement.ts module.
"""

import ast
from typing import Any

from ..lang.estree.parser import ESTreeParser


class Statement:
    """Statement compiler and parser for Nucleoid expressions."""

    @staticmethod
    def compile(statement_string: str) -> list[Any]:
        """
        Compile a statement string into executable AST nodes.
        
        Args:
            statement_string: The statement string to compile
            
        Returns:
            List of AST nodes ready for execution
        """
        try:
            # Use the ESTree parser to parse JavaScript-like syntax
            parser = ESTreeParser()
            nodes = parser.parse(statement_string)
            return nodes
        except Exception:
            # Fallback to Python AST parsing for simple expressions
            try:
                python_ast = ast.parse(statement_string, mode='eval')
                return [python_ast]
            except SyntaxError:
                # Try parsing as a statement instead of expression
                try:
                    python_ast = ast.parse(statement_string, mode='exec')
                    return python_ast.body
                except SyntaxError as parse_error:
                    raise SyntaxError(f"Failed to parse statement: {statement_string}") from parse_error
