from typing import Any
from .dollar import Dollar
# from ...nuc.variable import VARIABLE
# from .dollar_expression import dollar_expression
# from ..ast.identifier import Identifier as DollarIdentifier
# from ...nuc.node import NODE


def build(name: Any, value: Any) -> "DollarVARIABLE":
    """Build a DollarVARIABLE statement."""
    statement = DollarVARIABLE()
    statement.nme = name
    statement.val = value
    return statement


class DollarVARIABLE(Dollar):
    """Represents a variable declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.val: Any = None

    def before(self, scope: Any) -> None:
        """Prepare the variable statement by evaluating the value."""
        # Imports moved here to avoid circular dependencies
        from .dollar_expression import dollar_expression

        expression = dollar_expression(self.val)
        self.val = expression.run(scope)

    def run(self, scope: Any = None) -> "VARIABLE":
        """Execute the variable statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...nuc.variable import VARIABLE

        name = DollarIdentifier(self.nme)
        statement = VARIABLE(name)
        statement.name = name
        statement.value = self.val
        return statement


# Export the function as dollar_variable for consistency
dollar_variable = build
