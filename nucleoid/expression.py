"""
Expression processing for Nucleoid.

This module handles expression evaluation, traversal, and graph building,
corresponding to the TypeScript Expression.ts module.
"""

from collections.abc import Callable
from typing import Any, TypeVar

from .lang.ast.node import Node
from .types import ASTNode

T = TypeVar('T')


class Expression(Node):
    """Expression processor for AST nodes."""

    def __init__(self, node: ASTNode):
        """
        Initialize an Expression instance.
        
        Args:
            node: The AST node to process
        """
        super().__init__()
        self.node = node

    def map(self, fn: Callable[[ASTNode], T]) -> list[T]:
        """
        Map a function over all nodes in the expression.
        
        Args:
            fn: Function to apply to each node
            
        Returns:
            List of function results
        """
        return self._map_reduce(self.node, fn)

    def find(self, fn: Callable[[ASTNode], T]) -> T | None:
        """
        Find the first node that matches the predicate.
        
        Args:
            fn: Function to test each node
            
        Returns:
            First matching result or None
        """
        results = self._map_reduce(self.node, fn)
        return results[0] if results else None

    def resolve(self, scope: dict[str, Any] | None = None) -> Any:
        """
        Resolve the expression in the given scope.
        
        Args:
            scope: The evaluation scope
            
        Returns:
            The resolved value
        """
        scope = scope or {}
        converted_node = Node.convert(self.node)
        return converted_node.resolve(scope)

    def traverse(self, fn: Callable[[ASTNode], T]) -> list[T]:
        """
        Traverse the expression tree in order.
        
        Args:
            fn: Function to apply during traversal
            
        Returns:
            List of traversal results
        """
        return self._traverse_reduce(self.node, fn)

    def graph(self, scope: dict[str, Any], fn: Callable[[ASTNode], T]) -> list[T]:
        """
        Build dependency graph from the expression.
        
        Args:
            scope: The graph scope
            fn: Function to apply to graph nodes
            
        Returns:
            List of graph results
        """
        return self._graph_reduce(scope, self.node, fn)

    def _traverse_reduce(self, exp: ASTNode, fn: Callable[[ASTNode], T], acc: list[T] | None = None) -> list[T]:
        """
        Recursively traverse and reduce expression nodes.
        
        Args:
            exp: Expression node to traverse
            fn: Function to apply
            acc: Accumulator list
            
        Returns:
            Flattened list of results
        """
        if acc is None:
            acc = []

        exp_type = exp.get("type", "")

        if exp_type == "BinaryExpression":
            left = exp.get("left")
            right = exp.get("right")
            operator = exp.get("operator", "")

            if left and left.get("type") == "BinaryExpression":
                acc.append("(")

            if left:
                self._traverse_reduce(left, fn, acc)

            if left and left.get("type") == "BinaryExpression":
                acc.append(")")

            acc.append(operator)

            if right and right.get("type") == "BinaryExpression":
                acc.append("(")

            if right:
                self._traverse_reduce(right, fn, acc)

            if right and right.get("type") == "BinaryExpression":
                acc.append(")")

        elif exp_type in ["LogicalExpression"]:
            left = exp.get("left")
            right = exp.get("right")
            operator = exp.get("operator", "")

            if left:
                self._traverse_reduce(left, fn, acc)
            acc.append(operator)
            if right:
                self._traverse_reduce(right, fn, acc)

        elif exp_type == "UnaryExpression":
            operator = exp.get("operator", "")
            argument = exp.get("argument")

            acc.append(f"{operator} ")
            if argument:
                self._traverse_reduce(argument, fn, acc)
        else:
            ast_node = Node.convert(exp)
            result = fn(ast_node)

            if result is not None:
                acc.append(result)

        # Flatten nested lists
        return self._flatten(acc)

    def _map_reduce(self, exp: ASTNode, fn: Callable[[ASTNode], T], acc: list[T] | None = None) -> list[T]:
        """
        Recursively map and reduce expression nodes.
        
        Args:
            exp: Expression node to process
            fn: Function to apply
            acc: Accumulator list
            
        Returns:
            Flattened list of results
        """
        if acc is None:
            acc = []

        exp_type = exp.get("type", "")

        if exp_type in ["BinaryExpression", "LogicalExpression"]:
            left = exp.get("left")
            right = exp.get("right")

            if left:
                self._map_reduce(left, fn, acc)
            if right:
                self._map_reduce(right, fn, acc)

        elif exp_type == "UnaryExpression":
            argument = exp.get("argument")
            if argument:
                self._map_reduce(argument, fn, acc)
        else:
            ast_node = Node.convert(exp)
            result = fn(ast_node)

            if result is not None:
                acc.append(result)

        return self._flatten(acc)

    def _graph_reduce(self, scope: dict[str, Any], exp: ASTNode, fn: Callable[[ASTNode], T], acc: list[T] | None = None) -> list[T]:
        """
        Recursively build graph from expression nodes.
        
        Args:
            scope: The graph scope
            exp: Expression node to process
            fn: Function to apply to graph nodes
            acc: Accumulator list
            
        Returns:
            Flattened list of results
        """
        if acc is None:
            acc = []

        exp_type = exp.get("type", "")

        if exp_type in ["BinaryExpression", "LogicalExpression"]:
            left = exp.get("left")
            right = exp.get("right")

            if left:
                self._graph_reduce(scope, left, fn, acc)
            if right:
                self._graph_reduce(scope, right, fn, acc)

        elif exp_type == "UnaryExpression":
            argument = exp.get("argument")
            if argument:
                self._graph_reduce(scope, argument, fn, acc)
        else:
            ast_node = Node.convert(exp)

            # Build graph for this node
            graph_result = ast_node.graph(scope)
            if graph_result:
                graphed = [graph_result]
                flattened_graph = self._flatten(graphed)

                for item in flattened_graph:
                    if item is not None:
                        result = fn(item)
                        if result is not None:
                            acc.append(result)

        return self._flatten(acc)

    def _flatten(self, lst: list[Any]) -> list[Any]:
        """
        Flatten a nested list structure.
        
        Args:
            lst: List to flatten
            
        Returns:
            Flattened list
        """
        result = []
        for item in lst:
            if isinstance(item, list):
                result.extend(self._flatten(item))
            else:
                result.append(item)
        return result
