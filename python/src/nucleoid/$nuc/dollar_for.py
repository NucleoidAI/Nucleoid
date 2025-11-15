from typing import Any
from .dollar import Dollar
# from ...nuc.for_ import FOR
# from ...instruction import Instruction
# from ..ast.identifier import Identifier


def build(variable: Any, array: Any, statements: list[Dollar]) -> "DollarFOR":
    """Build a DollarFOR statement."""
    statement = DollarFOR()
    statement.var = variable
    statement.arr = array
    statement.stms = statements
    return statement


class DollarFOR(Dollar):
    """Represents a for loop statement."""

    def __init__(self) -> None:
        super().__init__()
        self.var: Any = None
        self.arr: Any = None
        self.stms: list[Dollar] = []

    def run(self, scope: Any) -> Any:
        """Execute the for loop statement."""
        # Imports moved here to avoid circular dependencies
        from ...nuc.for_ import FOR
        from ...instruction import Instruction
        from ..ast.identifier import Identifier

        statement = FOR()
        statement.variable = Identifier(self.var)
        statement.array = Identifier(self.arr)
        statement.statements = self.stms

        return Instruction(scope, statement, False, True, False, False)
