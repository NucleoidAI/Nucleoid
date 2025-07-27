"""Object expression AST node implementation."""
from typing import Any

from .node import Node


class Object(Node):
    """Object expression node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> dict[str, Any]:
        """Resolve object properties."""
        properties = self.node.get("properties", [])
        result = {}

        for prop in properties:
            key_node = Node.convert(prop.get("key", {}))
            value_node = Node.convert(prop.get("value", {}))

            key = key_node.resolve(scope)
            value = value_node.resolve(scope)

            result[str(key)] = value

        return result
