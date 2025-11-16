"""
VARIABLE runtime node class.
"""
from typing import Any, Dict
from .NODE import NODE


class VARIABLE(NODE):
    """Variable assignment node."""

    def __init__(self, value: NODE) -> None:
        """
        Initialize a VARIABLE instance.

        Args:
            value: The value node
        """
        super().__init__()
        self._value: NODE = value
        self.name: Any = None  # Set by subclasses

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        self._value.before(scope)

    def run(self, scope: Any) -> Dict[str, Any]:
        """
        Execute variable assignment.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with the assigned value
        """
        from ..state import state_instance

        evaluation = self._value.run(scope)
        value = state_instance.assign(scope, self.name, evaluation)

        return {"value": value}

    def graph(self, scope: Any) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope

        Returns:
            Graph result from value node
        """
        return self._value.graph(scope)
