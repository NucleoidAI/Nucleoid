"""
Identifier AST node implementation.
"""

import copy
from collections.abc import Iterator
from typing import Any, Optional

from ..estree.generator import append, root
from .node import Node


class Identifier(Node):
    """
    Identifier node for variable references and member expressions.
    
    Supports complex identifier chains like obj.prop.subprop and provides
    methods for traversal, resolution, and manipulation.
    """

    @staticmethod
    def get_types() -> list[str]:
        """Get supported AST node types for identifiers."""
        return ["Identifier", "MemberExpression", "ThisExpression"]

    @property
    def first(self) -> Optional['Identifier']:
        """
        Get the first (leftmost) identifier in a chain.
        
        For 'obj.prop.sub', returns 'obj'.
        For simple identifiers, returns self.
        """
        node_type = self.node.get("type", "")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return Identifier(self.node)
        elif node_type == "MemberExpression":
            r = root(self.node)
            if r.get("object"):
                return Identifier(r["object"])

        return None

    @first.setter
    def first(self, first: 'Identifier') -> None:
        """Set the first identifier in the chain."""
        current_first = self.first
        if not current_first:
            raise ValueError("Cannot set first on non-MemberExpression")

        # Clear current node and replace with first
        self.node.clear()
        self.node.update(first.node)

    @property
    def object(self) -> Optional['Identifier']:
        """Get the object part of a member expression."""
        node_type = self.node.get("type", "")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return None
        elif node_type == "MemberExpression":
            obj = self.node.get("object")
            if obj:
                return Identifier(obj)

        return None

    @object.setter
    def object(self, obj: 'Identifier') -> None:
        """Set the object part of a member expression."""
        current_object = self.object
        if not current_object:
            raise ValueError("Cannot set object on non-MemberExpression")

        self.node["object"] = obj.node

    @property
    def last(self) -> Optional['Identifier']:
        """
        Get the last (rightmost) identifier in a chain.
        
        For 'obj.prop.sub', returns 'sub'.
        For simple identifiers, returns self.
        """
        node_type = self.node.get("type", "")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return Identifier(self.node)
        elif node_type == "MemberExpression":
            prop = self.node.get("property")
            if prop:
                return Identifier(prop)

        return None

    @last.setter
    def last(self, last: 'Identifier') -> None:
        """Set the last identifier in the chain."""
        current_last = self.last
        if not current_last:
            raise ValueError("Cannot set last on non-MemberExpression")

        if self.node.get("type") == "MemberExpression":
            self.node["property"] = last.node
        else:
            # Replace entire node with last
            self.node.clear()
            self.node.update(last.node)

    def __str__(self) -> str:
        """Convert identifier to string representation."""
        node_type = self.node.get("type", "")

        if node_type == "Identifier":
            return self.node.get("name", "")
        elif node_type == "ThisExpression":
            return "this"
        elif node_type == "MemberExpression":
            obj = self.node.get("object", {})
            prop = self.node.get("property", {})
            obj_str = Identifier(obj).__str__()
            prop_str = Identifier(prop).__str__()
            return f"{obj_str}.{prop_str}"
        elif node_type == "Literal":
            return str(self.node.get("value", ""))

        return ""

    def resolve(self, scope: Any | None = None) -> Any:
        """
        Resolve identifier in the given scope.
        
        Args:
            scope: The scope object to resolve against
            
        Returns:
            The resolved AST node or value
        """
        if not scope:
            return self.node

        first = self.first
        if not first:
            return None

        # Check if this is a callback parameter
        if hasattr(scope, 'callback'):
            callback_strs = [str(arg) for arg in scope.callback]
            if str(first) in callback_strs:
                return self.node

        # Try to retrieve from scope
        scoped = scope.retrieve(self) if hasattr(scope, 'retrieve') else None
        if scoped:
            return scoped.resolve()

        # Handle computed member expressions
        node = copy.deepcopy(self.node)
        if node.get("computed"):
            prop = node.get("property", {})
            property_id = Identifier(copy.deepcopy(prop))
            resolved = property_id.resolve(scope)
            if resolved:
                node["property"] = resolved

        # Check if this references a graph entry
        # This would need to be implemented based on the graph system
        # For now, return the node
        return node

    def graph(self, scope: Any | None = None) -> Optional['Identifier']:
        """
        Get graph representation of this identifier.
        
        Args:
            scope: The scope object to check against
            
        Returns:
            Graph identifier or None
        """
        first = self.first
        if not first:
            return None

        # Check if this is a callback parameter
        if scope and hasattr(scope, 'callback'):
            callback_strs = [str(arg) for arg in scope.callback]
            if str(first) in callback_strs:
                return None

        # This would need integration with the graph system
        # For now, return a copy without builtins
        return self._remove_builtins()

    def _remove_builtins(self) -> 'Identifier':
        """Remove builtin properties like 'length' from identifier chain."""
        current = copy.deepcopy(self.node)

        # Remove .length property chains
        while (current.get("property", {}).get("name") == "length" and
               current.get("type") == "MemberExpression"):
            current = current.get("object", {})

        return Identifier(current)

    def walk(self) -> list[Node]:
        """Walk the identifier and return list of nodes."""
        return [self]

    def __iter__(self) -> Iterator[dict[str, Optional['Identifier']]]:
        """
        Iterator that yields left-right pairs traversing the identifier chain.
        
        Yields:
            Dict with 'left' and 'right' keys containing Identifier objects
        """
        parts = []
        left = self
        right = None

        while left.object:
            last = left.last
            if last:
                if right is None:
                    right = Identifier(last.node)
                else:
                    # Append last.node to right.node
                    right = Identifier(append(last.node, right.node))

                left = left.object
                parts.append({"left": left, "right": right})

        # Return iterator that yields in reverse order
        return iter(reversed(parts))
