from typing import Any
from .dollar import Dollar
# from ...nuc.throw import THROW
# from ..ast.dollar_expression import DollarExpression


def build(exception: Any) -> "DollarTHROW":
    """Build a DollarTHROW statement."""
    statement = DollarTHROW()
    statement.exc = exception
    return statement


class DollarTHROW(Dollar):
    """Represents a throw statement."""

    def __init__(self) -> None:
        super().__init__()
        self.exc: Any = None

    def run(self, scope: Any = None) -> Any:
        """Execute the throw statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.dollar_expression import DollarExpression
        from ...nuc.throw import THROW

        exception = DollarExpression(self.exc)
        return THROW(exception)
