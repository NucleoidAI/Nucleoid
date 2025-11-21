"""
BLOCK_INSTANCE runtime node class.
"""
from typing import Any, List, Dict
from .BLOCK import BLOCK


class BLOCK_INSTANCE(BLOCK):
    """Block instance for class-based execution."""

    def __init__(self, key: Any) -> None:
        """
        Initialize a BLOCK_INSTANCE.

        Args:
            key: The block instance key
        """
        super().__init__(str(key))
        self.instance: Dict[str, Any] = {}
        self.declaration: Any = None
        self.break_flag: bool = False
        self.type: str = "INSTANCE"

    def run(self, scope: Any) -> List[Any]:
        """
        Execute the block instance.

        Args:
            scope: The execution scope

        Returns:
            List of Instruction instances
        """
        scope.dollar_instance = self.instance

        if self.break_flag:
            self.statements = self.declaration.statements
            self.break_flag = False

        return super().run(scope)
