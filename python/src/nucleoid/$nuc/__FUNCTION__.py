from typing import Any, TYPE_CHECKING
from .__NUC__ import __NUC__

if TYPE_CHECKING:
    from typing import Optional


def build(name: Any, args: list[Any], block: "__BLOCK__") -> "__FUNCTION__":
    """Build a __FUNCTION__ statement."""
    statement = __FUNCTION__()
    statement.nme = name
    statement.args = args
    statement.blk = block
    return statement


class __FUNCTION__(__NUC__):
    """Represents a function declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.args: list[Any] = []
        self.blk: Optional["__BLOCK__"] = None

    def run(self, scope: Any = None) -> Any:
        """Execute the function declaration."""
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.function import FUNCTION

        name = __Identifier__(self.nme)
        statement = FUNCTION(name)
        statement.name = name
        statement.arguments = self.args
        statement.block = self.blk
        return statement
