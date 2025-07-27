"""New expression AST node implementation."""
from typing import Any

from .node import Node


class New(Node):
    """New expression node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve new expression."""
        # Placeholder implementation
        return {}  # Return empty object for now
