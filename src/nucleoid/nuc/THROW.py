"""
THROW runtime node class.
"""
from typing import Any
from .NODE import NODE


class THROW(NODE):
    """Throw exception statement node."""

    def __init__(self, exception: Any = None) -> None:
        """
        Initialize a THROW instance.

        Args:
            exception: The exception to throw
        """
        super().__init__()
        self.exception: Any = exception

    def before(self) -> None:
        """Execute before the main run."""
        pass

    def run(self, scope: Any) -> None:
        """
        Execute throw statement.

        Args:
            scope: The execution scope
        """
        from ..state import state_instance

        if self.exception and hasattr(self.exception, 'generate'):
            state_instance.throw(scope, self.exception.generate(scope))

    def graph(self) -> None:
        """Build dependency graph."""
        pass
