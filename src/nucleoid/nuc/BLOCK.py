"""
BLOCK runtime node class.
"""
from typing import Any, List
from .NODE import NODE


class BLOCK(NODE):
    """Block statement node containing multiple statements."""

    def __init__(self, key: Any = None) -> None:
        """
        Initialize a BLOCK instance.

        Args:
            key: The block key identifier
        """
        super().__init__(key)
        self.statements: List[Any] = []
        self.skip: bool = False

    def run(self, scope: Any) -> List[Any]:
        """
        Execute the block statements.

        Args:
            scope: The execution scope

        Returns:
            List of Instruction instances
        """
        # Lazy import to avoid circular dependencies
        from ..Instruction import Instruction

        return [
            Instruction(scope, statement, None, None, None, None)
            for statement in self.statements
        ]
