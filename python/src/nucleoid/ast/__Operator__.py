"""
Operator AST node (binary, logical, unary).
"""
from typing import Any, List, Optional, Union, Dict
import copy
from .__Node__ import __Node__


class __Operator__(__Node__):
    """Operator node for binary, logical, and unary expressions."""
    
    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve operator expressions in the given scope.
        
        Args:
            scope: The scope to resolve in
            
        Returns:
            Resolved operator node
        """
        if scope:
            cloned = copy.deepcopy(self.node)
            _traverse_resolve(scope, cloned)
            return cloned
        else:
            return self.node
    
    def walk(self) -> List["__Node__"]:
        """
        Walk the operator tree.
        
        Returns:
            List of child nodes
        """
        return _traverse_walk(self.node)
    
    def graph(self, scope: Any) -> List["__Node__"]:
        """
        Build dependency graph for operator.
        
        Args:
            scope: The scope to analyze
            
        Returns:
            List of dependent nodes
        """
        return _traverse_graph(scope, self.node)


def _traverse_walk(node: Dict[str, Any], acc: Optional[List["__Node__"]] = None) -> List["__Node__"]:
    """
    Walk through operator tree recursively.
    
    Args:
        node: The node to walk
        acc: Accumulator list
        
    Returns:
        List of nodes
    """
    if acc is None:
        acc = []
    
    node_type = node.get("type")
    
    if node_type in ["BinaryExpression", "LogicalExpression"]:
        _traverse_walk(node.get("left", {}), acc)
        _traverse_walk(node.get("right", {}), acc)
    elif node_type in ["UnaryExpression", "UpdateExpression"]:
        _traverse_walk(node.get("argument", {}), acc)
    else:
        # Lazy import to avoid circular dependencies
        from .convert import AST
        acc.extend(AST.convert(node).walk())
    
    return acc


def _traverse_resolve(scope: Any, node: Dict[str, Any]) -> None:
    """
    Resolve operator tree recursively, modifying node in place.
    
    Args:
        scope: The scope to resolve in
        node: The node to resolve (modified in place)
    """
    node_type = node.get("type")
    
    if node_type in ["BinaryExpression", "LogicalExpression"]:
        _traverse_resolve(scope, node.get("left", {}))
        _traverse_resolve(scope, node.get("right", {}))
    elif node_type in ["UnaryExpression", "UpdateExpression"]:
        _traverse_resolve(scope, node.get("argument", {}))
    else:
        # Lazy import to avoid circular dependencies
        from .convert import AST
        ast = AST.convert(copy.deepcopy(node))
        resolved = ast.resolve(scope)
        
        # Clear and update node in place
        node.clear()
        node.update(resolved)


def _traverse_graph(scope: Any, node: Dict[str, Any], acc: Optional[List["__Node__"]] = None) -> List["__Node__"]:
    """
    Build graph from operator tree recursively.
    
    Args:
        scope: The scope to analyze
        node: The node to graph
        acc: Accumulator list
        
    Returns:
        List of dependent nodes
    """
    if acc is None:
        acc = []
    
    node_type = node.get("type")
    
    if node_type in ["BinaryExpression", "LogicalExpression"]:
        _traverse_graph(scope, node.get("left", {}), acc)
        _traverse_graph(scope, node.get("right", {}), acc)
    elif node_type in ["UnaryExpression", "UpdateExpression"]:
        _traverse_graph(scope, node.get("argument", {}), acc)
    else:
        # Lazy import to avoid circular dependencies
        from .convert import AST
        acc.extend(AST.convert(node).graph(scope))
    
    return acc


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Operator__:
    """
    Builder function for creating operator instances.
    
    Args:
        node: The operator node
        
    Returns:
        An __Operator__ instance
    """
    return __Operator__(node)


__operator__ = build
