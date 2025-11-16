from typing import Any
from .__ import __


def build(name: Any, value: Any) -> "__VARIABLE__":
    """Build a __VARIABLE__ statement."""
    statement = __VARIABLE__()
    statement.nme = name
    statement.val = value
    return statement


class __VARIABLE__(__):
    """Represents a variable declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.val: Any = None

    def before(self, scope: Any) -> None:
        """Prepare the variable statement by evaluating the value."""
        from .__EXPRESSION__ import __expression__
        expression = __expression__(self.val)
        self.val = expression.run(scope)

    def run(self, scope: Any = None) -> "VARIABLE":
        """Execute the variable statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.variable import VARIABLE
        name = __Identifier__(self.nme)
        statement = VARIABLE(name)
        statement.name = name
        statement.value = self.val
        return statement


__variable__ = build
