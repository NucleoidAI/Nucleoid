"""Call expression AST node implementation."""
from typing import Any

from .node import Node


class Call(Node):
    """Function call expression node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve function call."""
        callee_node = Node.convert(self.node.get("callee", {}))
        arguments = self.node.get("arguments", [])

        # Resolve the function
        func = callee_node.resolve(scope)

        # Resolve arguments
        args = [Node.convert(arg).resolve(scope) for arg in arguments]

        # Call the function if it's callable
        if callable(func):
            try:
                return func(*args)
            except Exception:
                return None

        return None
