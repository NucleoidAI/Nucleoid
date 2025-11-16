"""
PROPERTY runtime node class.
"""
from typing import Any, Dict, Optional
from .NODE import NODE


class PROPERTY(NODE):
    """Property assignment node."""

    def __init__(self) -> None:
        """Initialize a PROPERTY instance."""
        super().__init__()
        self.object: Any = None
        self.name: Any = None
        self.value: Any = None

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        if self.value:
            self.value.before(scope)

    def run(self, scope: Any) -> Optional[Dict[str, Any]]:
        """
        Execute property assignment.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with value or None
        """
        from .. import graph, state
        from ..lang.ast import __Identifier__
        from ..lang.estree.estree import append
        from .REFERENCE import REFERENCE

        evaluation = self.value.run(scope) if self.value else None

        # Determine target object
        ref = self.object.value if hasattr(self.object, 'value') else None
        if isinstance(ref, REFERENCE):
            target = graph.retrieve(ref.link)
        else:
            target = self.object

        # Build variable path
        appended = append(target.resolve().node, self.name.node)
        variable = __Identifier__(appended)

        if not evaluation:
            state.state_instance.delete(scope, str(variable))
            return None

        assigned = state.state_instance.assign(scope, str(variable), evaluation)
        return {"value": assigned}

    def graph(self, scope: Any) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope

        Returns:
            Graph result from value
        """
        from .. import graph
        from .REFERENCE import REFERENCE

        # Determine target object
        ref = self.object.value if hasattr(self.object, 'value') else None
        if isinstance(ref, REFERENCE):
            target = graph.retrieve(ref.link)
        else:
            target = self.object

        target.properties[str(self.name)] = self
        return self.value.graph(scope) if self.value else None
