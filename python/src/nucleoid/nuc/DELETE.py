"""
DELETE runtime node class.
"""
from typing import Any, Dict, List, Optional
from .NODE import NODE


class DELETE:
    """Delete statement node."""

    def __init__(self) -> None:
        """Initialize a DELETE instance."""
        self.variable: Optional[Any] = None

    def before(self) -> None:
        """Execute before the main run."""
        pass

    def run(self, scope: Any) -> Dict[str, Any]:
        """
        Execute delete operation.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions and value
        """
        from ..Instruction import Instruction
        from .. import graph, state

        if self.variable:
            key = self.variable.key
            state.state_instance.delete(scope, key)

            result_list: List[Instruction] = []

            node = graph.retrieve(key)
            for next_node_key in node.next:
                statement = graph.retrieve(next_node_key)
                result_list.append(
                    Instruction(scope.root, statement, False, True, False, False)
                )

            return {"next": result_list, "value": True}
        else:
            return {"value": False}

    def before_graph(self) -> None:
        """Execute before graph operations."""
        pass

    def graph(self) -> None:
        """Build dependency graph."""
        if not self.variable:
            return

        from .. import graph

        node = graph.retrieve(self.variable.key)

        # Remove previous connections
        for key in list(node.previous.keys()):
            if key in node.next:
                del node.next[key]

        # Create empty node
        empty = NODE(node.key)

        # Transfer next connections
        for key in list(node.next.keys()):
            empty.next[key] = node.next[key]
            del node.next[key]

        # Delete from graph
        name = node.name
        if hasattr(node, 'object') and node.object and hasattr(node.object, 'properties'):
            del node.object.properties[name]

        del graph.graph_instance[node.key]
        graph.graph_instance[node.key] = empty
