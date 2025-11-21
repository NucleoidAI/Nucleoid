"""
Context module for loading and running definitions.
"""
from typing import Any, Dict, List, Optional


class ContextItem:
    """Represents a context item with definition and options."""

    def __init__(self, definition: str, options: Optional[Dict[str, Any]] = None) -> None:
        """
        Initialize a context item.

        Args:
            definition: Code definition
            options: Execution options
        """
        self.definition: str = definition
        self.options: Dict[str, Any] = options or {}


class Context:
    """Manages context loading and execution."""

    def __init__(self) -> None:
        """Initialize context with empty list."""
        self._context: List[ContextItem] = []

    def load(self, context: List[Dict[str, Any]]) -> None:
        """
        Load context items.

        Args:
            context: List of context items with definition and options
        """
        items = [
            ContextItem(
                definition=item.get('definition', ''),
                options=item.get('options')
            )
            for item in context
        ]
        self._context.extend(items)

    def run(self, context: Optional[List[Dict[str, Any]]] = None) -> None:
        """
        Run all context definitions.

        Args:
            context: Optional additional context items to load
        """
        # Lazy import to avoid circular dependency
        from . import nucleoid

        if context:
            self.load(context)

        for item in self._context:
            nucleoid.run(item.definition, item.options)

        self._context = []


# Create singleton instance
context_instance = Context()


def load(context: List[Dict[str, Any]]) -> None:
    """
    Load context items to the singleton instance.

    Args:
        context: List of context items with definition and options
    """
    context_instance.load(context)


def run(context: Optional[List[Dict[str, Any]]] = None) -> None:
    """
    Run all context definitions from the singleton instance.

    Args:
        context: Optional additional context items to load
    """
    context_instance.run(context)
