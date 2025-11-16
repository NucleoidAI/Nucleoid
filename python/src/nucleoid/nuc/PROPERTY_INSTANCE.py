"""
PROPERTY_INSTANCE runtime node class.
"""
from typing import Any
from .PROPERTY import PROPERTY


class PROPERTY_INSTANCE(PROPERTY):
    """Property instance for class-based property assignments."""

    def __init__(self, key: Any = None) -> None:
        """
        Initialize a PROPERTY_INSTANCE.

        Args:
            key: The property instance key
        """
        super().__init__()
        self.key = str(key) if key else ""

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope

        Raises:
            ValueError: If instance is missing in scope
        """
        dollar_instance = getattr(scope, 'dollar_instance', None)

        if not dollar_instance:
            raise ValueError("Declaration missing instance in scope")

        # Process value tokens to replace class name with instance
        if hasattr(self.value, 'tokens'):
            tokens = self.value.tokens

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
                    if (hasattr(dollar_instance, 'class_ref') and
                        hasattr(dollar_instance.class_ref, 'name') and
                        hasattr(identifier, 'first')):
                        if str(dollar_instance.class_ref.name) == str(identifier.first):
                            identifier.first = dollar_instance.resolve()

            tokens.traverse(process_node)

        super().before(scope)
