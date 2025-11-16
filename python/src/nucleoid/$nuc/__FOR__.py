from typing import Any
from .__NUC__ import __NUC__


def build(variable: Any, array: Any, statements: list[__NUC__]) -> "__FOR__":
    """Build a __FOR__ statement."""
    statement = __FOR__()
    statement.var = variable
    statement.arr = array
    statement.stms = statements
    return statement


class __FOR__(__NUC__):
    """Represents a for loop statement."""

    def __init__(self) -> None:
        super().__init__()
        self.var: Any = None
        self.arr: Any = None
        self.stms: list[__NUC__] = []

    def run(self, scope: Any) -> Any:
        """Execute the for loop statement."""
        from ...nuc.for_ import FOR
        from ...instruction import Instruction
        from ..ast.__identifier__ import __Identifier__

        statement = FOR()
        statement.variable = __Identifier__(self.var)
        statement.array = __Identifier__(self.arr)
        statement.statements = self.stms
        return Instruction(scope, statement, False, True, False, False)
