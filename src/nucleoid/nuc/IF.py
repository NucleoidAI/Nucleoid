"""
IF runtime node class.
"""
from typing import Any, Dict, List, Optional
import copy
from .NODE import NODE


class IF(NODE):
    """If conditional statement node."""

    def __init__(self) -> None:
        """Initialize an IF instance."""
        super().__init__()
        self.condition: Any = None
        self.true: Any = None
        self.false: Optional[Any] = None

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        if self.condition:
            self.condition.before(scope)

    def run(self, scope: Any) -> Dict[str, List[Any]]:
        """
        Execute if statement.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions
        """
        from ..Scope import Scope
        from ..Instruction import Instruction
        from ..state import state_instance

        local_scope = Scope(scope)

        # Evaluate condition
        if hasattr(scope, 'block') and scope.block and getattr(scope.block, 'skip', False):
            condition = self.condition.run(scope, True)
        else:
            condition = self.condition.run(scope, True)

        # Execute based on condition
        if state_instance.expression(scope, condition):
            true_statement = copy.deepcopy(self.true)
            return {
                "next": [
                    Instruction(local_scope, true_statement, True, True, False, False),
                    Instruction(local_scope, true_statement, False, False, True, True),
                ]
            }
        elif self.false:
            false_statement = copy.deepcopy(self.false)
            return {
                "next": [
                    Instruction(local_scope, false_statement, True, True, False, False),
                    Instruction(local_scope, false_statement, False, False, True, True),
                ]
            }

        return {"next": []}

    def graph(self, scope: Any) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope

        Returns:
            Graph result from condition
        """
        return self.condition.graph(scope)
