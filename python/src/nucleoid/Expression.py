"""
Expression class for AST expression manipulation.
"""
from typing import Any, Callable, Dict, List, Optional, TypeVar, Union

T = TypeVar('T')

# Type definitions
BinaryExpression = Dict[str, Any]
LogicalExpression = Dict[str, Any]
UnaryExpression = Dict[str, Any]
ASTNode = Dict[str, Any]


class Expression:
    """Represents an expression with traversal and manipulation methods."""

    def __init__(self, node: ASTNode) -> None:
        """
        Initialize an Expression.

        Args:
            node: AST node representing the expression
        """
        from .lang.ast import __Node__

        self.node: ASTNode = node
        self.__Node__ = __Node__

    def map(self, fn: Callable[[Any], T]) -> List[T]:
        """
        Map function over all expression nodes.

        Args:
            fn: Function to apply to each AST node

        Returns:
            List of results
        """
        return _map_reduce(self.node, fn)

    def find(self, fn: Callable[[Any], T]) -> Optional[T]:
        """
        Find first node matching function.

        Args:
            fn: Function to apply to each AST node

        Returns:
            First matching result or None
        """
        # TODO Optimize this with own reduce
        results = _map_reduce(self.node, fn)
        return results[0] if results else None

    def resolve(self, scope: Any) -> Any:
        """
        Resolve expression in scope.

        Args:
            scope: Execution scope

        Returns:
            Resolved value
        """
        return self.__Node__.convert(self.node).resolve(scope)

    def traverse(self, fn: Callable[[Any], T]) -> List[T]:
        """
        Traverse expression with operators.

        Args:
            fn: Function to apply to each AST node

        Returns:
            List of results including operators
        """
        return _traverse_reduce(self.node, fn)

    def graph(self, scope: Any, fn: Callable[[Any], T]) -> List[T]:
        """
        Graph expression dependencies.

        Args:
            scope: Execution scope
            fn: Function to apply to each graphed node

        Returns:
            List of graph results
        """
        return _graph_reduce(scope, self.node, fn)


def _traverse_reduce(
    exp: ASTNode,
    fn: Callable[[Any], T],
    acc: Optional[List[Any]] = None
) -> List[T]:
    """
    Traverse expression tree with operators.

    Args:
        exp: AST node to traverse
        fn: Function to apply to each node
        acc: Accumulator list

    Returns:
        Flattened list of results
    """
    from .lang.ast import __Node__

    if acc is None:
        acc = []

    if exp.get('type') == 'BinaryExpression':
        left = exp.get('left')
        right = exp.get('right')
        operator = exp.get('operator')

        if left and left.get('type') == 'BinaryExpression':
            acc.append('(')

        _traverse_reduce(left, fn, acc)

        if left and left.get('type') == 'BinaryExpression':
            acc.append(')')

        acc.append(operator)

        if right and right.get('type') == 'BinaryExpression':
            acc.append('(')

        _traverse_reduce(right, fn, acc)

        if right and right.get('type') == 'BinaryExpression':
            acc.append(')')

    elif exp.get('type') == 'LogicalExpression':
        left = exp.get('left')
        right = exp.get('right')
        operator = exp.get('operator')

        _traverse_reduce(left, fn, acc)
        acc.append(operator)
        _traverse_reduce(right, fn, acc)

    elif exp.get('type') == 'UnaryExpression':
        operator = exp.get('operator')
        argument = exp.get('argument')

        acc.append(f"{operator} ")
        _traverse_reduce(argument, fn, acc)

    else:
        ast = __Node__.convert(exp)
        curr = fn(ast)

        if curr is not None:
            acc.append(curr)

    return _flatten(acc)


def _map_reduce(
    exp: ASTNode,
    fn: Callable[[Any], T],
    acc: Optional[List[T]] = None
) -> List[T]:
    """
    Map function over expression nodes.

    Args:
        exp: AST node to traverse
        fn: Function to apply to each node
        acc: Accumulator list

    Returns:
        Flattened list of results
    """
    from .lang.ast import __Node__

    if acc is None:
        acc = []

    if exp.get('type') in ['BinaryExpression', 'LogicalExpression']:
        left = exp.get('left')
        right = exp.get('right')

        _map_reduce(left, fn, acc)
        _map_reduce(right, fn, acc)

    elif exp.get('type') == 'UnaryExpression':
        argument = exp.get('argument')
        _map_reduce(argument, fn, acc)

    else:
        ast = __Node__.convert(exp)
        curr = fn(ast)

        if curr is not None:
            acc.append(curr)

    return _flatten(acc)


def _graph_reduce(
    scope: Any,
    exp: ASTNode,
    fn: Callable[[Any], T],
    acc: Optional[List[T]] = None
) -> List[T]:
    """
    Graph expression dependencies.

    Args:
        scope: Execution scope
        exp: AST node to traverse
        fn: Function to apply to graphed nodes
        acc: Accumulator list

    Returns:
        Flattened list of results
    """
    from .lang.ast import __Node__

    if acc is None:
        acc = []

    if exp.get('type') in ['BinaryExpression', 'LogicalExpression']:
        left = exp.get('left')
        right = exp.get('right')

        _graph_reduce(scope, left, fn, acc)
        _graph_reduce(scope, right, fn, acc)

    elif exp.get('type') == 'UnaryExpression':
        argument = exp.get('argument')
        _graph_reduce(scope, argument, fn, acc)

    else:
        ast = __Node__.convert(exp)
        graphed = [ast.graph(scope)]

        for item in _flatten(graphed):
            if item is not None:
                curr = fn(item)
                if curr is not None:
                    acc.append(curr)

    return _flatten(acc)


def _flatten(lst: List[Any]) -> List[Any]:
    """
    Flatten nested lists to any depth.

    Args:
        lst: List to flatten

    Returns:
        Flattened list
    """
    result = []
    for item in lst:
        if isinstance(item, list):
            result.extend(_flatten(item))
        else:
            result.append(item)
    return result
