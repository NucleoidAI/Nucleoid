"""
Graph module for managing dependency graphs.
This is a stub implementation that will be expanded later.
"""

from typing import Any


class Graph:
    """Manages dependency graph for Nucleoid"""

    def __init__(self) -> None:
        self._nodes: dict[str, Any] = {}
        self._edges: list[tuple[str, str]] = []

    def clear(self) -> None:
        """Clear the dependency graph"""
        self._nodes.clear()
        self._edges.clear()

    def add_node(self, node_id: str, data: Any = None) -> None:
        """Add a node to the graph"""
        self._nodes[node_id] = data

    def add_edge(self, from_node: str, to_node: str) -> None:
        """Add an edge between two nodes"""
        self._edges.append((from_node, to_node))


# Singleton instance
_graph = Graph()


def clear() -> None:
    """Clear the graph"""
    _graph.clear()


def add_node(node_id: str, data: Any = None) -> None:
    """Add a node to the graph"""
    _graph.add_node(node_id, data)


def add_edge(from_node: str, to_node: str) -> None:
    """Add an edge between two nodes"""
    _graph.add_edge(from_node, to_node)
