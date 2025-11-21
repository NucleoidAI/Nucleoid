"""
BLOCK_CLASS runtime node class.
"""
from typing import Any, List, Optional, Dict
from .NODE import NODE
import uuid


class BLOCK_CLASS(NODE):
    """Block class for class-based statement execution."""

    def __init__(self, key: Any) -> None:
        """
        Initialize a BLOCK_CLASS.

        Args:
            key: The block class key
        """
        super().__init__(str(key) if not isinstance(key, str) else key)
        self.statements: List[Any] = []
        self.type: str = "CLASS"
        self.class_ref: Any = None  # Reference to the class
        self.id: str = ""

    def run(self, scope: Any) -> Optional[List[NODE]]:
        """
        Execute the block class.

        Args:
            scope: The execution scope

        Returns:
            List of NODE instances or None
        """
        # Lazy imports
        from .BLOCK_INSTANCE import BLOCK_INSTANCE
        from ..Instruction import Instruction
        from ..Scope import Scope

        # Get instances to process
        if hasattr(scope, 'dollar_instance') and scope.dollar_instance:
            instances = [scope.dollar_instance]
        else:
            instances = list(self.class_ref.instances.values()) if self.class_ref else []

        instructions: List[NODE] = []

        for instance in instances:
            statement = BLOCK_INSTANCE(uuid.uuid4())
            statement.id = str(uuid.uuid4())
            statement.class_ref = self.class_ref
            statement.instance = instance
            statement.statements = self.statements
            statement.declaration = self

            parent_scope = scope if hasattr(scope, 'block') else None
            instance_scope = Scope(parent_scope, statement)
            instance_scope.dollar_instance = self

            instructions.append(
                Instruction(
                    instance_scope,
                    statement,
                    lambda: None,
                    lambda: None,
                    None,
                    None
                )
            )

            instructions.append(
                Instruction(
                    instance_scope,
                    statement,
                    lambda: None,
                    lambda: None,
                    None,
                    None
                )
            )

        return instructions if instructions else None

    def graph(self) -> None:
        """Build dependency graph."""
        if self.class_ref:
            self.class_ref.declarations[self.id] = self
