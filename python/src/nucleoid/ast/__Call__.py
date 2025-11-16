"""
Call expression AST node.
"""
from typing import Any, List, Optional, Union, Dict
import copy
from .__Node__ import __Node__


class __Call__(__Node__):
    """Call expression node."""

    @property
    def first(self) -> Optional["__Identifier__"]:
        """
        Get the first identifier from the callee.

        Returns:
            The first identifier or None
        """
        from .__Identifier__ import __Identifier__

        callee = _root_callee(self.node)
        if not callee:
            return None

        identifier = __Identifier__(callee)
        return identifier.first

    @property
    def object(self) -> Optional["__Identifier__"]:
        """
        Get the object identifier from the callee.

        Returns:
            The object identifier or None
        """
        from .__Identifier__ import __Identifier__

        callee = _root_callee(self.node)
        if not callee:
            return None

        identifier = __Identifier__(callee)
        return identifier.object

    @property
    def last(self) -> Optional["__Identifier__"]:
        """
        Get the last identifier from the callee.

        Returns:
            The last identifier or None
        """
        from .__Identifier__ import __Identifier__

        root = _root_callee(self.node)
        if not root:
            return None

        identifier = __Identifier__(root)
        return identifier.last

    @property
    def function(self) -> "__Identifier__":
        """
        Get the function identifier from the callee.

        Returns:
            The function identifier

        Raises:
            ValueError: If callee is None
        """
        from .__Identifier__ import __Identifier__

        root = _root_callee(self.node)
        if not root:
            raise ValueError("Function callee is null or undefined")

        return __Identifier__(root)

    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve the call expression in the given scope.

        Args:
            scope: The scope to resolve in

        Returns:
            Resolved call node
        """
        if scope:
            # Lazy imports to avoid circular dependencies
            from ...graph import retrieve
            from ...nuc import FUNCTION
            from ...__nuc__ import __CALL__
            from ...lib.serialize import serialize
            from ...Expression import Expression

            clone = copy.deepcopy(self)
            first = clone.first

            if first and isinstance(retrieve(first), FUNCTION):
                # Process the call and serialize the result
                from ...stack import process

                value = process(
                    [__CALL__(self.node.get("callee"), self.node.get("arguments", []))],
                    scope
                )["value"]

                json_str = serialize(value, "state")
                ast_node = json.loads(json_str)
                expression = Expression(ast_node)
                return expression.resolve(scope)
            else:
                if first and first.type != "Literal":
                    resolved = first.resolve(scope)
                    root_node = _find_root(clone.node)

                    if resolved:
                        if root_node.get("object"):
                            root_node["object"] = resolved
                        else:
                            root_node["callee"] = resolved

                _resolve_arguments(scope, clone.node)

                return clone.node
        else:
            return self.node

    def graph(self, scope: Any) -> List["__Node__"]:
        """
        Build dependency graph for the call.

        Args:
            scope: The scope to analyze

        Returns:
            List of dependent nodes
        """
        result: List["__Node__"] = []

        if self.first:
            first_graph = self.first.graph(scope)
            result.extend(first_graph)

        arg_graphs = _traverse_callee(self.node, lambda callee: _graph_arguments(callee, scope))

        for graphs in arg_graphs:
            if isinstance(graphs, list):
                for item in graphs:
                    if isinstance(item, __Node__):
                        result.append(item)

        return result

    def walk(self) -> List["__Node__"]:
        """
        Walk the call expression tree.

        Returns:
            List of nodes
        """
        result: List["__Node__"] = []

        if self.first:
            first_walk = self.first.walk()
            result.extend(first_walk)

        arg_walks = _traverse_callee(self.node, lambda callee: _walk_arguments(callee))

        for walks in arg_walks:
            if isinstance(walks, list):
                for item in walks:
                    if isinstance(item, __Node__):
                        result.append(item)

        return result


def _find_root(node: Dict[str, Any]) -> Dict[str, Any]:
    """
    Find the root of a call/member expression chain.

    Args:
        node: The node to search

    Returns:
        The root node
    """
    current = node

    while True:
        obj_type = current.get("object", {}).get("type")
        callee_type = current.get("callee", {}).get("type")

        if obj_type in ["MemberExpression", "CallExpression"]:
            current = current["object"]
        elif callee_type in ["MemberExpression", "CallExpression"]:
            current = current["callee"]
        else:
            break

    return current


def _root_callee(node: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """
    Find the root callee of a call expression.

    Args:
        node: The call expression node

    Returns:
        The root callee or None
    """
    current = node
    callee = node.get("callee")

    if not callee:
        return None

    while True:
        obj_type = current.get("object", {}).get("type")
        callee_type = current.get("callee", {}).get("type")

        if callee_type:
            callee = current["callee"]

        if obj_type in ["MemberExpression", "CallExpression"]:
            current = current["object"]
        elif callee_type in ["MemberExpression", "CallExpression"]:
            current = current["callee"]
        else:
            break

    return callee


def _traverse_callee(node: Dict[str, Any], func) -> List[Any]:
    """
    Traverse a call expression chain and apply a function.

    Args:
        node: The node to traverse
        func: Function to apply to each callee

    Returns:
        List of results
    """
    current = node
    acc: List[Any] = []

    while True:
        node_type = current.get("type")
        obj_type = current.get("object", {}).get("type")
        callee_type = current.get("callee", {}).get("type")

        if current.get("callee"):
            acc.append(func(current))

        if node_type in ["MemberExpression", "CallExpression"] or \
           obj_type in ["MemberExpression", "CallExpression"] or \
           callee_type in ["MemberExpression", "CallExpression"]:
            current = current.get("object") or current.get("callee")
        else:
            break

    return acc


def _graph_arguments(callee: Dict[str, Any], scope: Any) -> List["__Node__"]:
    """
    Build graph for call arguments.

    Args:
        callee: The callee node
        scope: The scope to analyze

    Returns:
        List of dependent nodes
    """
    arguments = callee.get("arguments")
    if not arguments:
        return []

    result = []
    for arg in arguments:
        node = __Node__.convert(arg)
        if node:
            result.extend(node.graph(scope))

    return result


def _walk_arguments(callee: Dict[str, Any]) -> List["__Node__"]:
    """
    Walk call arguments.

    Args:
        callee: The callee node

    Returns:
        List of nodes
    """
    arguments = callee.get("arguments")
    if not arguments:
        return []

    result = []
    for arg in arguments:
        node = __Node__.convert(arg)
        if node:
            result.extend(node.walk())

    return result


def _resolve_arguments(scope: Any, node: Dict[str, Any]) -> None:
    """
    Resolve arguments in a call expression chain.

    Args:
        scope: The scope to resolve in
        node: The node to modify (modified in place)
    """
    current = node

    while True:
        node_type = current.get("type")
        obj_type = current.get("object", {}).get("type")

        if node_type in ["MemberExpression", "CallExpression"] or \
           obj_type in ["MemberExpression", "CallExpression"]:

            if current.get("callee") and current.get("arguments"):
                current["arguments"] = [
                    __Node__.convert(arg).resolve(scope) if __Node__.convert(arg) else arg
                    for arg in current["arguments"]
                ]

            current = current.get("callee") or current.get("object")
        else:
            break


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Call__:
    """
    Builder function for creating call instances.

    Args:
        node: The call node

    Returns:
        A __Call__ instance
    """
    return __Call__(node)


__call__ = build


# Register the CallExpression type
__Node__.register("CallExpression", __Call__)
