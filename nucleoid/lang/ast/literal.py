"""
Literal AST node implementation.
"""

from typing import Any

from .node import Node


class Literal(Node):
    """Literal value node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """Return the literal value."""
        return self.node.get("value")
