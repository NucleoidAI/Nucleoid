"""
Function AST node with scope handling.
"""
from typing import Any, List, Optional, Union, Dict
import copy
from .__Node__ import __Node__


class __Function__(__Node__):
    """Function node with parameter and body handling."""

    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve the function in the given scope.

        Args:
            scope: The scope to resolve in

        Returns:
            Resolved function node
        """
        if scope:
            from .__Identifier__ import __Identifier__

            cloned = copy.deepcopy(self.node)
            params = cloned.get("params", [])
            scope.callback = [__Identifier__(param) for param in params]
            _resolve_node(scope, cloned.get("body", {}))
            scope.callback = []
            return cloned
        else:
            return self.node

    def graph(self, scope: Any) -> List["__Identifier__"]:
        """
        Build dependency graph for the function.

        Args:
            scope: The scope to analyze

        Returns:
            List of dependent identifiers
        """
        from .__Identifier__ import __Identifier__

        params = self.node.get("params", [])
        scope.callback = [__Identifier__(param) for param in params]
        result = _graph_node(scope, self.node.get("body", {}))
        scope.callback = []
        return result


def _resolve_node(scope: Any, node: Dict[str, Any]) -> None:
    """
    Resolve a node recursively based on its type.

    Args:
        scope: The scope to resolve in
        node: The node to resolve (modified in place)
    """
    node_type = node.get("type")

    if node_type == "VariableDeclaration":
        for declaration in node.get("declarations", []):
            _resolve_node(scope, declaration.get("init", {}))

    elif node_type == "BlockStatement":
        for statement in node.get("body", []):
            _resolve_node(scope, statement)

    elif node_type == "ExpressionStatement":
        _resolve_node(scope, node.get("expression", {}))

    elif node_type == "AssignmentExpression":
        _resolve_node(scope, node.get("left", {}))
        _resolve_node(scope, node.get("right", {}))

    elif node_type == "IfStatement":
        _resolve_node(scope, node.get("test", {}))
        _resolve_node(scope, node.get("consequent", {}))
        if node.get("alternate"):
            _resolve_node(scope, node["alternate"])

    elif node_type == "ReturnStatement":
        _resolve_node(scope, node.get("argument", {}))

    else:
        _resolve_identifier(scope, node)


def _resolve_identifier(scope: Any, node: Dict[str, Any]) -> None:
    """
    Resolve identifiers in a node.

    Args:
        scope: The scope to resolve in
        node: The node to resolve (modified in place)
    """
    from .__Identifier__ import __Identifier__

    node_type = node.get("type")

    if node_type in ["BinaryExpression", "LogicalExpression"]:
        _resolve_identifier(scope, node.get("left", {}))
        _resolve_identifier(scope, node.get("right", {}))
    else:
        if node_type in ["Identifier", "MemberExpression"]:
            identifier = __Identifier__(copy.deepcopy(node))
            resolved = identifier.resolve(scope)

            # Update node in place
            node.clear()
            node.update(resolved)


def _graph_node(scope: Any, node: Dict[str, Any]) -> List["__Identifier__"]:
    """
    Build graph for a node recursively.

    Args:
        scope: The scope to analyze
        node: The node to graph

    Returns:
        List of dependent identifiers
    """
    node_type = node.get("type")

    if node_type == "VariableDeclaration":
        for declaration in node.get("declarations", []):
            _graph_node(scope, declaration.get("init", {}))

    elif node_type == "BlockStatement":
        for statement in node.get("body", []):
            _graph_node(scope, statement)

    elif node_type == "ExpressionStatement":
        _graph_node(scope, node.get("expression", {}))

    elif node_type == "AssignmentExpression":
        _graph_node(scope, node.get("left", {}))
        _graph_node(scope, node.get("right", {}))

    elif node_type == "IfStatement":
        _graph_node(scope, node.get("test", {}))
        _graph_node(scope, node.get("consequent", {}))
        if node.get("alternate"):
            _graph_node(scope, node["alternate"])

    elif node_type == "ReturnStatement":
        _graph_node(scope, node.get("argument", {}))

    else:
        return _graph_identifier(scope, node)

    return []


def _graph_identifier(scope: Any, node: Dict[str, Any], acc: Optional[List["__Identifier__"]] = None) -> List["__Identifier__"]:
    """
    Build graph for identifiers in a node.

    Args:
        scope: The scope to analyze
        node: The node to graph
        acc: Accumulator list

    Returns:
        List of dependent identifiers
    """
    from .__Identifier__ import __Identifier__

    if acc is None:
        acc = []

    node_type = node.get("type")

    if node_type in ["BinaryExpression", "LogicalExpression"]:
        _graph_identifier(scope, node.get("left", {}), acc)
        _graph_identifier(scope, node.get("right", {}), acc)
    else:
        if node_type in ["Identifier", "MemberExpression"]:
            identifier = __Identifier__(node)
            graph = identifier.graph(scope)

            if graph:
                acc.extend(graph)

    return acc


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Function__:
    """
    Builder function for creating function instances.

    Args:
        node: The function node

    Returns:
        A __Function__ instance
    """
    return __Function__(node)


__function__ = build
