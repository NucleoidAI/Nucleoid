"""If statement construct for Nucleoid AI."""

from typing import Any, Optional
from dataclasses import dataclass


@dataclass
class IfStatement:
    """Represents an if statement."""
    
    test: Any  # Condition to test
    consequent: Any  # Statement to execute if true
    alternate: Optional[Any] = None  # Statement to execute if false
    
    def execute(self, context: Any = None) -> Any:
        """Execute the if statement."""
        # In a full implementation, this would evaluate the condition
        # For now, we'll execute the consequent
        if hasattr(self.consequent, 'execute'):
            return self.consequent.execute(context)
        return self.consequent
    
    def __str__(self) -> str:
        """String representation of the if statement."""
        if self.alternate:
            return f"If({self.test}) then {self.consequent} else {self.alternate}"
        return f"If({self.test}) then {self.consequent}"