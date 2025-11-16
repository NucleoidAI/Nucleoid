"""
LET runtime node class.
"""
from typing import Any, Dict, Optional
from .NODE import NODE


class LET(NODE):
    """Let/const variable declaration node."""

    def __init__(self, name: Any = None, value: Any = None) -> None:
        """
        Initialize a LET instance.

        Args:
            name: Variable name
            value: Variable value
        """
        super().__init__()
        self.name: Any = name
        self.value: Any = value
        self.reassign: bool = False

    def before(self) -> None:
        """Execute before the main run."""
        pass

    def run(self, scope: Any) -> Optional[Dict[str, Any]]:
        """
        Execute let declaration.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with value or None

        Raises:
            TypeError: If attempting to reassign constant variable
        """
        if self.reassign:
            instance = scope.retrieve_graph(self.name.first) if hasattr(self.name, 'first') else None

            if instance and getattr(instance, 'constant', False):
                raise TypeError("Assignment to constant variable.")

        evaluation = self.value.run(scope, False, False) if self.value else None

        if not evaluation:
            return None

        value = scope.assign(self.name, evaluation, self.reassign)
        return {"value": value}

    def before_graph(self, scope: Any) -> None:
        """
        Execute before graph operations.

        Args:
            scope: The execution scope
        """
        if not self.reassign:
            scope.graph[self.name] = self

    def graph(self, scope: Any) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope

        Returns:
            Graph result from value
        """
        if hasattr(scope, 'block') and scope.block is not None:
            return self.value.graph(scope) if self.value else None
        return None
