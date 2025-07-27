"""Function AST node implementation."""
from collections.abc import Callable
from typing import Any

from .node import Node


class Function(Node):
    """Function expression node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> Callable:
        """Resolve function."""
        # Placeholder implementation - return a lambda
        return lambda *args, **kwargs: None
