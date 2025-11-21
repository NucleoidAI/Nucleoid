"""
LET_INSTANCE runtime node class.
"""
from typing import Any
from .LET import LET


class LET_INSTANCE(LET):
    """Let instance for class-based variable declarations."""

    def __init__(self) -> None:
        """Initialize a LET_INSTANCE."""
        super().__init__()
        self.class_ref: Any = None
        self.instance: Any = None

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        # Process value tokens to replace class name with instance
        def process_node(node: Any) -> None:
            identifiers = []
            walk_result = node.walk()

            # Flatten walk result
            if isinstance(walk_result, list):
                for item in walk_result:
                    if isinstance(item, list):
                        identifiers.extend(item)
                    else:
                        identifiers.append(item)
            else:
                identifiers.append(walk_result)

            for identifier in identifiers:
                if (hasattr(identifier, 'first') and
                    hasattr(self, 'class_ref') and
                    hasattr(self.class_ref, 'name')):
                    if str(identifier.first) == str(self.class_ref.name):
                        identifier.first = self.instance.resolve()

        if hasattr(self.value, 'tokens') and hasattr(self.value.tokens, 'traverse'):
            self.value.tokens.traverse(process_node)

        super().before(scope)
