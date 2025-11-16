from typing import Any
from .__ import __


def build(exception: Any) -> "__THROW__":
    """Build a __THROW__ statement."""
    statement = __THROW__()
    statement.exc = exception
    return statement


class __THROW__(__):
    """Represents a throw statement."""

    def __init__(self) -> None:
        super().__init__()
        self.exc: Any = None

    def run(self, scope: Any = None) -> Any:
        """Execute the throw statement."""
        from ..ast.__expression__ import __Expression__
        from ...nuc.throw import THROW
        exception = __Expression__(self.exc)
        return THROW(exception)
