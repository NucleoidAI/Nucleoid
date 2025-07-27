"""Template literal AST node implementation."""
from typing import Any

from .node import Node


class Template(Node):
    """Template literal node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> str:
        """Resolve template literal."""
        quasis = self.node.get("quasis", [])
        expressions = self.node.get("expressions", [])

        result = ""

        for i, quasi in enumerate(quasis):
            # Add the quasi (literal part)
            result += quasi.get("value", {}).get("cooked", "")

            # Add the expression if there's one
            if i < len(expressions):
                expr_node = Node.convert(expressions[i])
                expr_value = expr_node.resolve(scope)
                result += str(expr_value) if expr_value is not None else ""

        return result
