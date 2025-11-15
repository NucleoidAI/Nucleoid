from typing import Any
from .dollar_expression import dollar_expression, DollarExpression
from .dollar import Dollar
# from ..ast.dollar_identifier import DollarIdentifier
# from ...nuc.alias import ALIAS


def build(alias: Any, name: str, value: "DollarExpression") -> "DollarALIAS":
    """Build a DollarALIAS statement."""
    statement = DollarALIAS()
    statement.als = alias
    statement.nme = name
    statement.val = value
    return statement


class DollarALIAS(Dollar):
    """Represents an alias statement."""

    def __init__(self) -> None:
        super().__init__()
        self.als: Any = None
        self.nme: str = ""
        self.val: Any = None

    def before(self, scope: Any) -> None:
        """Prepare the alias statement by evaluating the value expression."""
        expression = dollar_expression(self.val)
        self.val = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the alias statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.dollar_identifier import DollarIdentifier
        from ...nuc.alias import ALIAS

        name = DollarIdentifier(self.nme)
        statement = ALIAS()
        statement.alias = DollarIdentifier(self.als)
        statement.name = name
        statement.value = self.val
        return statement
