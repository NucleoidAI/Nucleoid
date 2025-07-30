"""Return statement construct for Nucleoid AI."""

from typing import Any, Optional
from dataclasses import dataclass


@dataclass
class Return:
    """Represents a return statement."""
    
    value: Optional[Any] = None  # Value to return
    
    def execute(self, context: Any = None) -> Any:
        """Execute the return statement."""
        return self.value
    
    def __str__(self) -> str:
        """String representation of the return."""
        return f"Return({self.value})"