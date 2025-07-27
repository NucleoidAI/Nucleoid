"""Array AST node implementation."""
from typing import Any

from .node import Node


class Array(Node):
    """Array expression node."""

    def __init__(self, elements: list[Node]):
        super().__init__()
        self.elements = elements
        self.node = {"type": "ArrayExpression", "elements": [elem.node for elem in elements]}

    def resolve(self, scope: dict[str, Any] | None = None) -> list[Any]:
        """Resolve array elements."""
        return [elem.resolve(scope) for elem in self.elements]
