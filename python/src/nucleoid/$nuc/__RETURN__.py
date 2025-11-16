from typing import Any
from .__ import __


def build(statement: __) -> "__RETURN__":
    """Build a __RETURN__ statement."""
    return_statement = __RETURN__()
    return_statement.stm = statement
    return return_statement


class __RETURN__(__):
    """Represents a return statement."""

    def __init__(self) -> None:
        super().__init__()
        self.stm: __ = None

    def run(self, scope: Any = None) -> Any:
        """Execute the return statement."""
        from ...nuc.return_ import RETURN
        return RETURN(self.stm)
