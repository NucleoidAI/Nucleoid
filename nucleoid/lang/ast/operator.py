"""Operator AST node implementation."""
from typing import Any

from .node import Node


class Operator(Node):
    """Operator expression node."""

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve operator expression."""
        node_type = self.node.get("type", "")

        if node_type == "BinaryExpression":
            return self._resolve_binary(scope)
        elif node_type == "LogicalExpression":
            return self._resolve_logical(scope)
        elif node_type == "UnaryExpression":
            return self._resolve_unary(scope)
        else:
            return self.node

    def _resolve_binary(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve binary expression."""
        left_node = Node.convert(self.node.get("left", {}))
        right_node = Node.convert(self.node.get("right", {}))
        operator = self.node.get("operator", "")

        left_val = left_node.resolve(scope)
        right_val = right_node.resolve(scope)

        # Basic arithmetic and comparison operators
        if operator == "+":
            return left_val + right_val
        elif operator == "-":
            return left_val - right_val
        elif operator == "*":
            return left_val * right_val
        elif operator == "/":
            return left_val / right_val if right_val != 0 else None
        elif operator == "==":
            return left_val == right_val
        elif operator == "!=":
            return left_val != right_val
        elif operator == "<":
            return left_val < right_val
        elif operator == ">":
            return left_val > right_val
        elif operator == "<=":
            return left_val <= right_val
        elif operator == ">=":
            return left_val >= right_val
        else:
            return None

    def _resolve_logical(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve logical expression."""
        left_node = Node.convert(self.node.get("left", {}))
        right_node = Node.convert(self.node.get("right", {}))
        operator = self.node.get("operator", "")

        left_val = left_node.resolve(scope)

        if operator == "&&":
            return left_val and right_node.resolve(scope)
        elif operator == "||":
            return left_val or right_node.resolve(scope)
        else:
            return None

    def _resolve_unary(self, scope: dict[str, Any] | None = None) -> Any:
        """Resolve unary expression."""
        argument_node = Node.convert(self.node.get("argument", {}))
        operator = self.node.get("operator", "")

        arg_val = argument_node.resolve(scope)

        if operator == "!":
            return not arg_val
        elif operator == "-":
            return -arg_val
        elif operator == "+":
            return +arg_val
        else:
            return arg_val
