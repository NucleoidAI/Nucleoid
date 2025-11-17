"""
Graph module for managing dependency graphs.
"""
from typing import Any, Dict, Optional, Union


class Graph:
    """Manages dependency graph for Nucleoid."""

    def __init__(self) -> None:
        """Initialize graph with classes node."""
        self.dollar: Dict[str, Any] = {
            'classes': {
                'name': 'classes'
            }
        }

    def retrieve(self, identifier: Union[str, Any]) -> Optional[Any]:
        """
        Retrieve a node from the graph.

        Args:
            identifier: String identifier or object with generate() method

        Returns:
            Node value or None
        """
        if isinstance(identifier, str):
            return self.dollar.get(identifier)
        else:
            # Object with generate() method
            if hasattr(identifier, 'generate'):
                key = identifier.generate()
                return self.dollar.get(key)
            return None

    def clear(self) -> None:
        """Clear the dependency graph."""
        self.dollar.clear()
        self.dollar['classes'] = {'name': 'classes'}


# Create singleton instance
graph_instance = Graph()


def retrieve(identifier: Union[str, Any]) -> Optional[Any]:
    """
    Retrieve a node from the graph.

    Args:
        identifier: String identifier or object with generate() method

    Returns:
        Node value or None
    """
    return graph_instance.retrieve(identifier)


def clear() -> None:
    """Clear the graph."""
    graph_instance.clear()
