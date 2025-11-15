from typing import Any, TYPE_CHECKING
from .dollar import Dollar
# from ...nuc.function import FUNCTION
# from ..ast.identifier import Identifier
# from .dollar_block import DollarBLOCK

if TYPE_CHECKING:
    from typing import Optional


def build(name: Any, args: list[Any], block: "DollarBLOCK") -> "DollarFUNCTION":
    """Build a DollarFUNCTION statement."""
    statement = DollarFUNCTION()
    statement.nme = name
    statement.args = args
    statement.blk = block
    return statement


class DollarFUNCTION(Dollar):
    """Represents a function declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.args: list[Any] = []
        self.blk: Optional["DollarBLOCK"] = None

    def run(self, scope: Any = None) -> Any:
        """Execute the function declaration."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier
        from ...nuc.function import FUNCTION

        name = Identifier(self.nme)
        statement = FUNCTION(name)
        statement.name = name
        statement.arguments = self.args
        statement.block = self.blk
        return statement
