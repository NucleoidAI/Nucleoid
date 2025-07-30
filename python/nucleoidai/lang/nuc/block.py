"""Block construct for Nucleoid AI."""

from typing import Any, List
from dataclasses import dataclass


@dataclass
class Block:
    """Represents a block of statements."""
    
    statements: List[Any]
    
    def execute(self, context: Any = None) -> Any:
        """Execute all statements in the block."""
        result = None
        for statement in self.statements:
            if hasattr(statement, 'execute'):
                result = statement.execute(context)
            else:
                result = statement
        return result
    
    def __str__(self) -> str:
        """String representation of the block."""
        return f"Block({len(self.statements)} statements)"