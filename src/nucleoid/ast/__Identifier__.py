"""
Identifier AST node with first/last/object accessors.
"""
from typing import Any, List, Optional, Union, Dict, Iterator, Tuple
import copy
from .__Node__ import __Node__


class __Identifier__(__Node__):
    """Identifier node with member expression support."""

    types: List[str] = ["Identifier", "MemberExpression", "ThisExpression"]

    @property
    def first(self) -> Optional["__Identifier__"]:
        """
        Get the first (leftmost) identifier in a member expression chain.

        Returns:
            The first identifier or None
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return __Identifier__(self.node)
        elif node_type == "MemberExpression":
            # Lazy import to avoid circular dependencies
            from ..lang.estree.estree import root
            return __Identifier__(root(self.node).get("object"))
        else:
            return None

    @first.setter
    def first(self, first: "__Identifier__") -> None:
        """
        Set the first identifier in a member expression chain.

        Args:
            first: The new first identifier
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression"]:
            self.node.clear()
            self.node.update(first.node)
        elif node_type == "MemberExpression":
            from ..lang.estree.estree import root
            root(self.node)["object"] = first.node

    @property
    def object(self) -> Optional["__Identifier__"]:
        """
        Get the object part of a member expression.

        Returns:
            The object identifier or None
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return None
        elif node_type == "MemberExpression":
            return __Identifier__(self.node.get("object"))
        else:
            return None

    @object.setter
    def object(self, obj: "__Identifier__") -> None:
        """
        Set the object part of a member expression.

        Args:
            obj: The new object identifier
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression"]:
            self.node.clear()
            self.node.update(obj.node)
        elif node_type == "MemberExpression":
            self.node["object"] = obj.node

    @property
    def last(self) -> Optional["__Identifier__"]:
        """
        Get the last (rightmost) identifier in a member expression chain.

        Returns:
            The last identifier or None
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression", "Literal"]:
            return __Identifier__(self.node)
        elif node_type == "MemberExpression":
            return __Identifier__(self.node.get("property"))
        else:
            return None

    @last.setter
    def last(self, last: "__Identifier__") -> None:
        """
        Set the last identifier in a member expression chain.

        Args:
            last: The new last identifier
        """
        node_type = self.node.get("type")

        if node_type in ["Identifier", "ThisExpression"]:
            self.node.clear()
            self.node.update(last.node)
        elif node_type == "MemberExpression":
            self.node["property"] = last.node

    def resolve(self, scope: Any) -> Optional[Dict[str, Any]]:
        """
        Resolve the identifier in the given scope.

        Args:
            scope: The scope to resolve in

        Returns:
            Resolved node or None
        """
        if scope:
            first = self.first

            if not first:
                return None

            # Check if identifier is a callback parameter
            callback_args = [str(arg) for arg in scope.callback] if hasattr(scope, 'callback') else []
            if first.generate(scope) in callback_args:
                return self.node

            # Try to retrieve from scope
            if hasattr(scope, 'retrieve'):
                scoped = scope.retrieve(self)
                if scoped:
                    return scoped.resolve()

            node = copy.deepcopy(self.node)

            # Handle computed member expressions
            if node.get("computed"):
                property_node = copy.deepcopy(node.get("property", {}))
                property_id = __Identifier__(property_node)
                resolved = property_id.resolve(scope)

                if resolved:
                    node["property"] = resolved

            # Check if first identifier is in graph
            from ...graph import retrieve
            if retrieve(first):
                state = {
                    "type": "Identifier",
                    "name": "state"
                }

                from ..lang.estree.estree import append
                return append(state, node)
            else:
                return node
        else:
            return self.node

    def graph(self, scope: Any) -> List["__Identifier__"]:
        """
        Build dependency graph for the identifier.

        Args:
            scope: The scope to analyze

        Returns:
            List of dependent identifiers
        """
        if self.first and hasattr(scope, 'callback'):
            callback_args = [str(arg) for arg in scope.callback]
            if self.first.generate(scope) in callback_args:
                return []

        from ...graph import retrieve
        first = retrieve(self.first)

        if first:
            return [_remove_builtins(self)]

        return []

    def walk(self) -> List["__Node__"]:
        """
        Walk the identifier tree.

        Returns:
            List containing this identifier
        """
        return [self]

    def __iter__(self) -> Iterator[Tuple["__Identifier__", "__Identifier__"]]:
        """
        Iterate through member expression chain.

        Yields:
            Tuples of (left, right) identifiers
        """
        from ..lang.estree.estree import append

        result_list: List[Tuple["__Identifier__", "__Identifier__"]] = []
        left: "__Identifier__" = self
        right: Optional["__Identifier__"] = None

        while left.object:
            last_node = left.last.node if left.last else {}
            right_node = right.node if right else {}

            appended = append(last_node, right_node)
            right = __Identifier__(appended)
            left = left.object
            result_list.append((left, right))

        # Return iterator in reverse order
        return iter(reversed(result_list))


def _remove_builtins(identifier: "__Identifier__") -> "__Identifier__":
    """
    Remove built-in properties like 'length' from identifier chain.

    Args:
        identifier: The identifier to process

    Returns:
        Identifier with built-ins removed
    """
    current = copy.deepcopy(identifier.node)

    # Skip properties named 'length'
    while current and isinstance(current.get("property"), dict):
        if current.get("property", {}).get("name") == "length":
            current = current.get("object")
        else:
            break

    return __Identifier__(current)


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Identifier__:
    """
    Builder function for creating identifier instances.

    Args:
        node: The identifier node

    Returns:
        An __Identifier__ instance
    """
    return __Identifier__(node)


__identifier__ = build
