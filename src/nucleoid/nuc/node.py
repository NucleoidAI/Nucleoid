"""Base runtime node."""

from __future__ import annotations


class Node:
    """Minimal base class for runtime statement nodes."""

    def before(self, scope: object | None = None) -> None:
        """Prepare the node before execution."""

    def run(self, scope: object | None = None) -> object:
        """Execute the node and return its result."""
        raise NotImplementedError

    def graph(self, scope: object | None = None) -> object:
        """Build graph metadata for the node."""
        return None
