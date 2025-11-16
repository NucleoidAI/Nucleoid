from typing import Any
from .__ import __


def build(name: Any, methods: list["__FUNCTION__"] = None) -> "__CLASS__":
    """Build a __CLASS__ statement."""
    if methods is None:
        methods = []
    statement = __CLASS__()
    statement.nme = name
    statement.mths = methods
    return statement


class __CLASS__(__):
    """Represents a class declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.mths: list[Any] = []

    def run(self, scope: Any = None) -> Any:
        """Execute the class declaration."""
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.class_ import CLASS

        name = __Identifier__(self.nme)
        statement = CLASS(f"${name}")
        statement.name = __Identifier__(f"${name}")
        statement.list = name

        methods = {}
        for method in self.mths:
            identifier = method.nme
            if hasattr(identifier, "name") and identifier.name == "constructor":
                identifier.name = "$constructor"
            method_name = identifier.name if hasattr(identifier, "name") else str(identifier)
            methods[method_name] = method

        statement.methods = methods
        return statement
