"""
Base NODE class for Nucleoid runtime nodes.
"""
from typing import Any, Dict, Optional


# Module-level sequence counter
_sequence = 0


class NODE:
    """Base class for all runtime node types."""

    def __init__(self, key: Optional[Any] = None) -> None:
        """
        Initialize a NODE instance.

        Args:
            key: The key identifier for this node
        """
        global _sequence

        if key is not None:
            self.key: str = str(key)
        else:
            self.key = ""

        self.next: Dict[str, Any] = {}
        self.previous: Dict[str, Any] = {}
        self.sequence: int = _sequence
        _sequence += 1

    def before(self, scope: Optional[Any] = None) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        pass

    def run(self, scope: Optional[Any] = None) -> Any:
        """
        Execute the node.

        Args:
            scope: The execution scope

        Returns:
            Execution result
        """
        pass

    def before_graph(self, scope: Optional[Any] = None) -> Any:
        """
        Execute before graph operations.

        Args:
            scope: The execution scope
        """
        pass

    def graph(self, scope: Optional[Any] = None) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope
        """
        pass

    def after(self, scope: Optional[Any] = None) -> None:
        """
        Execute after the main run.

        Args:
            scope: The execution scope
        """
        pass

    @staticmethod
    def register(key: str, node: "NODE") -> None:
        """
        Register a node in the graph.

        Args:
            key: The key to register under
            node: The node to register
        """
        # Lazy import to avoid circular dependencies
        from .. import transaction
        from ..graph import graph_instance as dollar

        transaction.register(dollar, key, node)

    @staticmethod
    def replace(source_key: str, target_node: "NODE") -> None:
        """
        Replace a node in the graph.

        Args:
            source_key: The key of the source node
            target_node: The target node to replace with
        """
        from .. import transaction
        from ..graph import graph_instance as dollar

        transaction.register(target_node.block, dollar[source_key].block)

        for node in dollar[source_key].next:
            transaction.register(target_node.next, node, dollar[source_key].next[node])
            transaction.register(dollar[source_key].next, node, None)

        for node in dollar[source_key].previous:
            transaction.register(dollar[node].next, source_key, None)

        transaction.register(dollar, source_key, target_node)

    @staticmethod
    def direct(source_key: str, target_key: str, target_node: "NODE") -> None:
        """
        Create a direct connection between nodes.

        Args:
            source_key: The source node key
            target_key: The target node key
            target_node: The target node
        """
        from .. import transaction
        from ..graph import graph_instance as dollar

        transaction.register(dollar[source_key].next, target_key, target_node)
        transaction.register(target_node.previous, source_key, dollar[source_key])
