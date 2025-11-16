"""
IF_INSTANCE runtime node class.
"""
from typing import Any
from .IF import IF


class IF_INSTANCE(IF):
    """If instance for class-based conditional execution."""

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

        # Process condition tokens to replace class name with instance
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

        if hasattr(self.condition, 'tokens') and hasattr(self.condition.tokens, 'traverse'):
            self.condition.tokens.traverse(process_node)

        self.key = f"if({self.condition.tokens})"

        super().before(scope)
