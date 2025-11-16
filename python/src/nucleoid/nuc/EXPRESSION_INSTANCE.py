"""
EXPRESSION_INSTANCE runtime node class.
"""
from typing import Any
from .EXPRESSION import EXPRESSION


class EXPRESSION_INSTANCE(EXPRESSION):
    """Expression instance for class-based execution."""

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        dollar_instance = getattr(scope, 'dollar_instance', None)

        if dollar_instance:
            def process_node(node: Any) -> None:
                identifiers = []
                walk_result = node.walk()
                # Flatten the result
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
                        str(identifier.first) == str(dollar_instance.class_ref.name)):
                        identifier.first = dollar_instance.resolve()

            if hasattr(self.tokens, 'traverse'):
                self.tokens.traverse(process_node)

        super().before(scope)
