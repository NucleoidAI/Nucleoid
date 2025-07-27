"""
Graph management for Nucleoid runtime.

This module provides graph storage and manipulation functionality for tracking
relationships between nodes in the Nucleoid system.
"""

from typing import Any

# Global graph storage - equivalent to TypeScript's $ export
graph_store: dict[str, Any] = {}


def retrieve(key: str) -> Any:
    """
    Retrieve a value from the graph store.
    
    Args:
        key: The key to retrieve
        
    Returns:
        The stored value, or None if not found
    """
    return graph_store.get(key)


def store(key: str, value: Any) -> None:
    """
    Store a value in the graph store.
    
    Args:
        key: The key to store under
        value: The value to store
    """
    graph_store[key] = value


def clear() -> None:
    """Clear all entries from the graph store."""
    graph_store.clear()


def keys() -> list:
    """Get all keys in the graph store."""
    return list(graph_store.keys())


def values() -> list:
    """Get all values in the graph store."""
    return list(graph_store.values())


def items() -> list:
    """Get all key-value pairs in the graph store."""
    return list(graph_store.items())
