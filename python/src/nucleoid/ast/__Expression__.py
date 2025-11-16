"""
Expression AST node with map/reduce/traverse operations.
"""
from typing import Any, Callable, List, Optional, Union, Dict
from .__Node__ import __Node__


class __Expression__(__Node__):
    """Expression node with traversal and mapping operations."""
    
    def map(self, fn: Callable[["__Node__", int], Any]) -> List[Any]:
        """
        Map a function over all nodes in the expression.
        
        Args:
            fn: Function to apply to each node
            
        Returns:
            List of results
        """
        return _map_reduce(self.node, fn)
    
    def find(self, fn: Callable[["__Node__", int], Any]) -> Any:
        """
        Find the first node matching the predicate.
        
        Args:
            fn: Predicate function
            
        Returns:
            First matching result or None
        """
        results = _map_reduce(self.node, fn)
        return results[0] if results else None
    
    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve the expression in the given scope.
        
        Args:
            scope: The scope to resolve in
            
        Returns:
            Resolved node
        """
        return __Node__.convert(self.node).resolve(scope)
    
    def traverse(self, fn: Callable[["__Node__"], Any]) -> List[Any]:
        """
        Traverse the expression tree.
        
        Args:
            fn: Function to apply during traversal
            
        Returns:
            List of traversal results
        """
        return _traverse_reduce(self.node, fn)
    
    def graph(self, scope: Any, fn: Optional[Callable[["__Node__"], Any]] = None) -> List[Any]:
        """
        Build a dependency graph.
        
        Args:
            scope: The scope to analyze
            fn: Optional function to apply to each graphed node
            
        Returns:
            List of graph results
        """
        if fn is None:
            fn = lambda x: x
        return _graph_reduce(scope, self.node, fn)


def _traverse_reduce(
    exp: Dict[str, Any],
    fn: Callable[["__Node__"], Any],
    acc: Optional[List[Any]] = None
) -> List[Any]:
    """
    Traverse and reduce an expression tree.
    
    Args:
        exp: The expression node
        fn: Function to apply
        acc: Accumulator list
        
    Returns:
        Flattened list of results
    """
    if acc is None:
        acc = []
    
    exp_type = exp.get("type")
    
    if exp_type == "BinaryExpression":
        left = exp.get("left", {})
        if left.get("type") == "BinaryExpression":
            acc.append("(")
        
        _traverse_reduce(left, fn, acc)
        
        if left.get("type") == "BinaryExpression":
            acc.append(")")
        
        acc.append(exp.get("operator"))
        
        right = exp.get("right", {})
        if right.get("type") == "BinaryExpression":
            acc.append("(")
        
        _traverse_reduce(right, fn, acc)
        
        if right.get("type") == "BinaryExpression":
            acc.append(")")
    
    elif exp_type == "LogicalExpression":
        _traverse_reduce(exp.get("left", {}), fn, acc)
        acc.append(exp.get("operator"))
        _traverse_reduce(exp.get("right", {}), fn, acc)
    
    elif exp_type == "UnaryExpression":
        acc.append(f"{exp.get('operator')} ")
        _traverse_reduce(exp.get("argument", {}), fn, acc)
    
    else:
        ast = __Node__.convert(exp)
        curr = fn(ast)
        if curr is not None:
            acc.append(curr)
    
    # Flatten the list
    return _flatten(acc)


def _map_reduce(
    exp: Dict[str, Any],
    fn: Callable[["__Node__", int], Any],
    acc: Optional[List[Any]] = None
) -> List[Any]:
    """
    Map and reduce over an expression tree.
    
    Args:
        exp: The expression node
        fn: Function to apply
        acc: Accumulator list
        
    Returns:
        Flattened list of results
    """
    if acc is None:
        acc = []
    
    exp_type = exp.get("type")
    
    if exp_type in ["BinaryExpression", "LogicalExpression"]:
        _map_reduce(exp.get("left", {}), fn, acc)
        _map_reduce(exp.get("right", {}), fn, acc)
    elif exp_type == "UnaryExpression":
        _map_reduce(exp.get("argument", {}), fn, acc)
    else:
        ast = __Node__.convert(exp)
        curr = fn(ast, len(acc))
        if curr is not None:
            acc.append(curr)
    
    return _flatten(acc)


def _graph_reduce(
    scope: Any,
    exp: Dict[str, Any],
    fn: Callable[["__Node__"], Any],
    acc: Optional[List[Any]] = None
) -> List[Any]:
    """
    Build a graph from an expression tree.
    
    Args:
        scope: The scope to analyze
        exp: The expression node
        fn: Function to apply
        acc: Accumulator list
        
    Returns:
        Flattened list of results
    """
    if acc is None:
        acc = []
    
    exp_type = exp.get("type")
    
    if exp_type in ["BinaryExpression", "LogicalExpression"]:
        _graph_reduce(scope, exp.get("left", {}), fn, acc)
        _graph_reduce(scope, exp.get("right", {}), fn, acc)
    elif exp_type == "UnaryExpression":
        _graph_reduce(scope, exp.get("argument", {}), fn, acc)
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
    Flatten a nested list to arbitrary depth.
    
    Args:
        lst: The list to flatten
        
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


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Expression__:
    """
    Builder function for creating expression instances.
    
    Args:
        node: The node to wrap
        
    Returns:
        An __Expression__ instance
    """
    return __Expression__(node)


__expression__ = build
