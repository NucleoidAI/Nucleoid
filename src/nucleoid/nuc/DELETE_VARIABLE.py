"""
DELETE_VARIABLE runtime node class.
"""
from .DELETE import DELETE


class DELETE_VARIABLE(DELETE):
    """Delete variable statement node."""

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
