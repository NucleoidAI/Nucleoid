from typing import Any
from .dollar import Dollar
# from ...nuc.return_ import RETURN


def build(statement: Dollar) -> "DollarRETURN":
    """Build a DollarRETURN statement."""
    return_statement = DollarRETURN()
    return_statement.stm = statement
    return return_statement


class DollarRETURN(Dollar):
    """Represents a return statement."""

    def __init__(self) -> None:
        super().__init__()
        self.stm: Dollar = None

    def run(self, scope: Any = None) -> Any:
        """Execute the return statement."""
        # Imports moved here to avoid circular dependencies
        from ...nuc.return_ import RETURN

        return RETURN(self.stm)
