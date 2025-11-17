from typing import Any
from .__NUC__ import __NUC__


def build(statement: Any) -> "__RETURN__":
    """Build a __RETURN__ statement."""
    return_statement = __RETURN__()
    return_statement.stm = statement
    return return_statement


class __RETURN__(__NUC__):
    """Represents a return statement."""

    def __init__(self) -> None:
        super().__init__()
        self.stm: Any = None

    def run(self, scope: Any = None) -> Any:
        """Execute the return statement."""
        from ...nuc.return_ import RETURN
        return RETURN(self.stm)
