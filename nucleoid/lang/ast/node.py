"""
Base AST Node class for Nucleoid expressions.

This module provides the base Node class and conversion utilities for
handling AST nodes, corresponding to the TypeScript Node.ts module.
"""

from abc import ABC
from typing import Any, Optional

from ...types import BaseNode
from ..estree.generator import ESTreeGenerator


class Node(ABC):
    """Base class for all AST nodes in Nucleoid."""

    def __init__(self, node: BaseNode | str | None = None):
        """
        Initialize a Node instance.
        
        Args:
            node: Either a BaseNode dictionary or string to parse
        """
        self.iof = self.__class__.__name__

        if node is None:
            self.node = {
                "type": "Literal",
                "value": None,
                "raw": "null",
            }
        elif isinstance(node, dict) and "type" in node:
            self.node = node
        elif isinstance(node, str):
            # Parse string using ESTree parser
            from ..estree.parser import ESTreeParser
            parser = ESTreeParser()
            parsed_nodes = parser.parse(node, False)
            self.node = parsed_nodes[0] if parsed_nodes else {"type": "Literal", "value": None}
        else:
            self.node = {"type": "Literal", "value": node}

    @property
    def type(self) -> str:
        """Get the node type."""
        return self.node.get("type", "Unknown")

    @property
    def first(self) -> Optional["Node"]:
        """Get the first child node."""
        return None

    @property
    def object(self) -> Optional["Node"]:
        """Get the object node."""
        return None

    @property
    def last(self) -> Optional["Node"]:
        """Get the last child node."""
        return None

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """
        Resolve the node in the given scope.
        
        Args:
            scope: The evaluation scope
            
        Returns:
            The resolved value
        """
        return self.node

    def generate(self, scope: dict[str, Any] | None = None) -> str:
        """
        Generate code string from this node.
        
        Args:
            scope: The generation scope
            
        Returns:
            Generated code string
        """
        resolved = self.resolve(scope)
        generator = ESTreeGenerator()
        return generator.generate(resolved)

    def graph(self, scope: dict[str, Any] | None = None) -> Optional["Node"]:
        """
        Build dependency graph from this node.
        
        Args:
            scope: The graph scope
            
        Returns:
            Graph node or None
        """
        return None

    def walk(self) -> list["Node"]:
        """
        Walk through child nodes.
        
        Returns:
            List of child nodes
        """
        return []

    def __str__(self) -> str:
        """String representation of the node."""
        return self.generate()

    @staticmethod
    def convert(node: BaseNode) -> "Node":
        """
        Convert a BaseNode to appropriate Node subclass.
        
        Args:
            node: The base node to convert
            
        Returns:
            Appropriate Node subclass instance
        """
        # Import specific node types here to avoid circular imports
        from .array import Array
        from .call import Call
        from .function import Function
        from .identifier import Identifier
        from .literal import Literal
        from .new import New
        from .object import Object
        from .operator import Operator
        from .template import Template

        node_type = node.get("type", "Unknown")

        if node_type == "Literal":
            return Literal(node)
        elif node_type in ["Identifier", "MemberExpression"]:
            return Identifier(node)
        elif node_type == "ArrayExpression":
            elements = [Node.convert(el) for el in node.get("elements", [])]
            return Array(elements)
        elif node_type == "NewExpression":
            return New(node)
        elif node_type == "ObjectExpression":
            return Object(node)
        elif node_type in ["FunctionExpression", "ArrowFunctionExpression"]:
            return Function(node)
        elif node_type == "CallExpression":
            return Call(node)
        elif node_type == "TemplateLiteral":
            return Template(node)
        else:
            return Operator(node)
