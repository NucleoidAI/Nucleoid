from typing import Any
from .__EXPRESSION__ import __EXPRESSION__, __expression__
from .__ import __


def build(alias: Any, name: str, value: "__EXPRESSION__") -> "__ALIAS__":
    """Build a __ALIAS__ statement."""
    statement = __ALIAS__()
    statement.als = alias
    statement.nme = name
    statement.val = value
    return statement


class __ALIAS__(__):
    """Represents an alias statement."""

    def __init__(self) -> None:
        super().__init__()
        self.als: Any = None
        self.nme: str = ""
        self.val: Any = None

    def before(self, scope: Any) -> None:
        """Prepare the alias statement by evaluating the value expression."""
        expression = __expression__(self.val)
        self.val = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the alias statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.alias import ALIAS

        name = __Identifier__(self.nme)
        statement = ALIAS()
        statement.alias = __Identifier__(self.als)
        statement.name = name
        statement.value = self.val
        return statement
