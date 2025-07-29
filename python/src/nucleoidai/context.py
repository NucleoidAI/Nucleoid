"""Context management for Nucleoid AI runtime."""

from typing import Any, Dict, List, Optional
from dataclasses import dataclass


@dataclass
class ContextItem:
    """A context item containing definition and options."""
    definition: str
    options: Dict[str, Any]


class ContextManager:
    """Manages runtime context for declarative functions and statements."""
    
    def __init__(self):
        self._context: List[ContextItem] = []
    
    def load(self, context: List[Dict[str, Any]]) -> None:
        """
        Load context items into the runtime.
        
        Args:
            context: List of context items with definition and options
        """
        context_items = [
            ContextItem(
                definition=item["definition"],
                options=item.get("options", {})
            )
            for item in context
        ]
        self._context.extend(context_items)
    
    def run(self, context: Optional[List[Dict[str, Any]]] = None) -> None:
        """
        Run all loaded context items.
        
        Args:
            context: Optional additional context to load and run
        """
        # Load additional context if provided
        if context:
            self.load(context)
        
        # Import nucleoid here to avoid circular imports
        from . import nucleoid
        
        # Execute all context items
        for item in self._context:
            nucleoid.run(item.definition, item.options)
        
        # Clear context after execution
        self._context = []
    
    def get_context(self) -> List[ContextItem]:
        """Get current context items."""
        return self._context.copy()
    
    def clear(self) -> None:
        """Clear all context items."""
        self._context = []