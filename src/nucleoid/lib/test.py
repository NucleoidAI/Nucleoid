"""
Test utility for clearing application state.
Used for test isolation and cleanup between test runs.
"""

from nucleoid import datastore, state, graph


def clear() -> None:
    """
    Clear all application state including state, graph, and datastore.
    This is useful for resetting the application between test runs.
    """
    state.clear()
    graph.clear()
    datastore.clear()
