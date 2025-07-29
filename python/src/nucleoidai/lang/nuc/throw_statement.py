"""Throw statement construct for Nucleoid AI."""

from typing import Any, Optional
from dataclasses import dataclass


@dataclass
class Throw:
    """Represents a throw/raise statement."""
    
    value: Optional[Any] = None  # Exception to throw
    
    def execute(self, context: Any = None) -> Any:
        """Execute the throw statement."""
        if self.value:
            raise Exception(str(self.value))
        else:
            raise Exception("Unspecified error")
    
    def __str__(self) -> str:
        """String representation of the throw."""
        return f"Throw({self.value})"