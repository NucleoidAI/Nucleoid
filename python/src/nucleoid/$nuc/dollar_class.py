from typing import Any
from .dollar import Dollar
# from ...nuc.class_ import CLASS
# from .dollar_function import DollarFUNCTION
# from ..ast.identifier import Identifier as DollarIdentifier


def build(name: Any, methods: list["DollarFUNCTION"] = None) -> "DollarCLASS":
    """Build a DollarCLASS statement."""
    if methods is None:
        methods = []
    statement = DollarCLASS()
    statement.nme = name
    statement.mths = methods
    return statement


class DollarCLASS(Dollar):
    """Represents a class declaration statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.mths: list[Any] = []

    def run(self, scope: Any = None) -> Any:
        """Execute the class declaration."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...nuc.class_ import CLASS

        name = DollarIdentifier(self.nme)

        statement = CLASS(f"${name}")
        statement.name = DollarIdentifier(f"${name}")
        statement.list = name

        # Build methods dictionary
        methods = {}
        for method in self.mths:
            identifier = method.nme

            # Rename constructor to $constructor
            if hasattr(identifier, "name") and identifier.name == "constructor":
                identifier.name = "$constructor"

            method_name = identifier.name if hasattr(identifier, "name") else str(identifier)
            methods[method_name] = method

        statement.methods = methods

        return statement
