"""Identifier AST node for Nucleoid AI."""

from typing import Any, Optional


class Identifier:
    """Represents an identifier in the AST."""
    
    def __init__(self, name: Any):
        if hasattr(name, 'name'):
            self.name = name.name
        else:
            self.name = str(name) if name is not None else ""
        self.type = "Identifier"
    
    def __str__(self) -> str:
        return self.name
    
    def __repr__(self) -> str:
        return f"Identifier({self.name!r})"