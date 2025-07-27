"""
Context management for the Nucleoid runtime.

This module manages the execution context, loading and running declarative statements
in the proper sequence. It corresponds to the TypeScript context.ts module.
"""

from typing import Any

from pydantic import BaseModel


class ContextItem(BaseModel):
    """A single context item containing a definition and options."""
    definition: str
    options: dict[str, Any]


# Global context storage
_context: list[ContextItem] = []


def load(context: list[dict[str, Any]]) -> None:
    """
    Load context items into the global context.
    
    Args:
        context: List of context items with definition and options
    """
    global _context

    # Convert dict items to ContextItem objects
    context_items = [
        ContextItem(definition=item["definition"], options=item.get("options", {}))
        for item in context
    ]

    _context.extend(context_items)


def run(context: list[dict[str, Any]] | None = None) -> None:
    """
    Run all loaded context items through the Nucleoid runtime.
    
    Args:
        context: Optional additional context items to load and run
    """
    global _context

    # Import here to avoid circular imports
    from . import run as nucleoid_run

    # Load additional context if provided
    if context:
        load(context)

    # Execute all context items
    for item in _context:
        nucleoid_run(item.definition, item.options)

    # Clear context after execution
    _context = []


def get_context() -> list[ContextItem]:
    """Get the current context items."""
    return _context.copy()


def clear_context() -> None:
    """Clear all context items."""
    global _context
    _context = []
