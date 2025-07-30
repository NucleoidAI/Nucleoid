"""For statement construct for Nucleoid AI."""

from typing import Any, List
from dataclasses import dataclass


@dataclass
class ForStatement:
    """Represents a for loop statement."""
    
    variable: Any  # Loop variable
    iterable: Any  # Object to iterate over
    body: List[Any]  # Loop body statements
    
    def execute(self, context: Any = None) -> Any:
        """Execute the for loop."""
        # In a full implementation, this would execute the loop
        result = None
        for statement in self.body:
            if hasattr(statement, 'execute'):
                result = statement.execute(context)
            else:
                result = statement
        return result
    
    def __str__(self) -> str:
        """String representation of the for loop."""
        return f"For({self.variable} in {self.iterable})"