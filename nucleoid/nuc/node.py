"""
Base NODE class for Nucleoid NUC (Nucleoid Unit of Computation) system.

This module provides the base NODE class that represents computation units in the
Nucleoid graph system with dependency tracking and lifecycle management.
"""

from typing import Any

from .. import transaction
from ..graph import graph_store

# Global sequence counter for node ordering
_sequence = 0


class NODE:
    """
    Base class for Nucleoid computation nodes.
    
    Each node has a unique key, maintains dependency relationships (next/previous),
    and provides lifecycle hooks for execution phases.
    """

    def __init__(self, key: Any) -> None:
        """
        Initialize a new NODE with the given key.
        
        Args:
            key: Unique identifier for this node (will be converted to string)
        """
        global _sequence

        self.key = str(key)
        self.next: dict[str, Any] = {}
        self.previous: dict[str, Any] = {}
        self.sequence = _sequence
        _sequence += 1

    def before(self, scope: Any | None = None) -> None:
        """
        Pre-execution hook. Override in subclasses.
        
        Args:
            scope: The execution scope
        """
        pass

    def run(self, scope: Any | None = None) -> Any:
        """
        Main execution method. Override in subclasses.
        
        Args:
            scope: The execution scope
            
        Returns:
            The result of execution
        """
        pass

    def before_graph(self, scope: Any | None = None) -> None:
        """
        Pre-graph processing hook. Override in subclasses.
        
        Args:
            scope: The execution scope
        """
        pass

    def graph(self, scope: Any | None = None) -> None:
        """
        Graph processing method. Override in subclasses.
        
        Args:
            scope: The execution scope
        """
        pass

    def after(self, scope: Any | None = None) -> None:
        """
        Post-execution hook. Override in subclasses.
        
        Args:
            scope: The execution scope
        """
        pass

    @staticmethod
    def register(key: str, node: 'NODE') -> None:
        """
        Register a node in the global graph.
        
        Args:
            key: The key to register the node under
            node: The node to register
        """
        transaction.register(graph_store, key, node)

    @staticmethod
    def replace(source_key: str, target_node: 'NODE') -> None:
        """
        Replace a node in the graph with another node.
        
        Args:
            source_key: Key of the node to replace
            target_node: Node to replace with
        """
        # Replace the block
        if source_key in graph_store and hasattr(graph_store[source_key], 'block'):
            transaction.register(target_node, 'block', graph_store[source_key].block)

        # Transfer next relationships
        if source_key in graph_store:
            source_node = graph_store[source_key]
            for node_key, node_ref in source_node.next.items():
                transaction.register(target_node.next, node_key, node_ref)
                transaction.register(source_node.next, node_key, None)

        # Update previous relationships
        if source_key in graph_store:
            source_node = graph_store[source_key]
            for node_key in source_node.previous:
                if node_key in graph_store:
                    transaction.register(graph_store[node_key].next, source_key, None)

        # Register the new node
        transaction.register(graph_store, source_key, target_node)

    @staticmethod
    def direct(source_key: str, target_key: str, target_node: 'NODE') -> None:
        """
        Create a direct relationship between two nodes.
        
        Args:
            source_key: Key of the source node
            target_key: Key of the target node
            target_node: The target node object
        """
        if source_key in graph_store:
            source_node = graph_store[source_key]
            transaction.register(source_node.next, target_key, target_node)
            transaction.register(target_node.previous, source_key, source_node)
