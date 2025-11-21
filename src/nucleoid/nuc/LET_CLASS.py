"""
LET_CLASS runtime node class.
"""
from typing import Any, Dict, Optional
import copy
from .NODE import NODE


class LET_CLASS(NODE):
    """Let class for class-based variable declarations."""

    def __init__(self) -> None:
        """Initialize a LET_CLASS instance."""
        super().__init__()
        self.name: Any = None
        self.value: Any = None
        self.class_ref: Any = None
        self.type: str = "CLASS"

    def before(self) -> None:
        """Execute before the main run."""
        pass

    def run(self, scope: Any) -> Optional[Dict[str, Any]]:
        """
        Execute let class.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next statement or None
        """
        from .LET_INSTANCE import LET_INSTANCE

        dollar_instance = getattr(scope, 'dollar_instance', None)

        if dollar_instance:
            statement = LET_INSTANCE()
            statement.class_ref = self.class_ref
            statement.instance = dollar_instance
            statement.name = self.name
            statement.value = copy.deepcopy(self.value)
            return {"next": statement}

        return None
