"""
LET_OBJECT runtime node class.
"""
from typing import Any, Dict
from .LET import LET


class LET_OBJECT(LET):
    """Let object for object-based variable declarations."""

    def __init__(self) -> None:
        """Initialize a LET_OBJECT."""
        super().__init__()
        self.object: Any = None

    def before(self) -> None:
        """Execute before the main run."""
        pass

    def run(self, scope: Any) -> Dict[str, str]:
        """
        Execute let object.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with value
        """
        from ..lang.Evaluation import Evaluation

        evaluation = Evaluation(self.object.key)
        value = scope.assign(self.name, f"state.{evaluation}")
        return {"value": value}

    def graph(self) -> None:
        """Build dependency graph."""
        pass
