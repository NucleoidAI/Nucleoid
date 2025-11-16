"""
DELETE_OBJECT runtime node class.
"""
from typing import Any, Dict, List
from .DELETE import DELETE


class DELETE_OBJECT(DELETE):
    """Delete object statement node."""

    def run(self, scope: Any) -> Dict[str, Any]:
        """
        Execute object delete operation.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions and value

        Raises:
            ReferenceError: If object has properties and cannot be deleted
        """
        from .. import graph, state

        node = graph.retrieve(self.variable.key)
        name = node.name

        # Check if object has properties
        if hasattr(node, 'properties') and len(node.properties) > 0:
            raise ReferenceError(f"Cannot delete object '{self.variable.key}'")

        if hasattr(node, 'object') and node.object:
            del node.object.properties[name]
        else:
            # Delete from class list
            list_name = str(node.class_ref.list)
            state.state_instance.delete(scope, f"{list_name}.{name}")

            # Find and remove from state list
            index = None
            for i, obj in enumerate(state.state_instance.dollar[list_name]):
                if obj.get('id') == node.key:
                    index = i
                    break

            if index is not None:
                state.state_instance.dollar[list_name].pop(index)

        return super().run(scope)

    def graph(self) -> None:
        """Build dependency graph."""
        from .. import graph

        node = graph.retrieve(self.variable.key)

        # Remove previous connections
        for key in list(node.previous.keys()):
            if key in node.next:
                del node.next[key]

        # Delete from graph
        del graph.graph_instance[node.key]
