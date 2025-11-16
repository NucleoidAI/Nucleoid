"""
IF_CLASS runtime node class.
"""
from typing import Any, Dict, List
import copy
import uuid as uuid_module
from .NODE import NODE


class IF_CLASS(NODE):
    """If class for class-based conditional execution."""

    def __init__(self) -> None:
        """Initialize an IF_CLASS instance."""
        super().__init__()
        self.condition: Any = None
        self.true: Any = None
        self.false: Any = None
        self.class_ref: Any = None
        self.type: str = "CLASS"

    def run(self, scope: Any) -> Dict[str, List[Any]]:
        """
        Execute if class.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions
        """
        from .IF_INSTANCE import IF_INSTANCE
        from ..Scope import Scope
        from ..Instruction import Instruction
        from ...__nuc__ import __BLOCK__

        # Get instances to process
        dollar_instance = getattr(scope, 'dollar_instance', None)
        if dollar_instance:
            instances = [dollar_instance]
        else:
            instances = list(self.class_ref.instances.values()) if self.class_ref else []

        statements: List[Any] = []

        for instance in instances:
            statement = IF_INSTANCE(str(uuid_module.uuid4()))
            statement.condition = copy.deepcopy(self.condition)
            statement.true = __BLOCK__(copy.deepcopy(self.true.stms))

            if self.false and hasattr(self.false, 'iof'):
                if self.false.iof == "$IF":
                    statement.false = copy.deepcopy(self.false)
                elif self.false.iof == "$BLOCK":
                    statement.false = __BLOCK__(copy.deepcopy(self.false.stms))

            instance_scope = Scope(scope)
            instance_scope.dollar_instance = instance

            statements.append(
                Instruction(instance_scope, statement, True, True, True, True)
            )

        return {"next": statements}

    def graph(self) -> None:
        """Build dependency graph."""
        if self.class_ref:
            self.class_ref.declarations[self.key] = self
