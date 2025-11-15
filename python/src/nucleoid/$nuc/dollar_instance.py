from typing import Any, Optional
from .dollar import Dollar
# from ..ast.identifier import Identifier as DollarIdentifier
# from ...nuc.class_ import CLASS
# from ...instruction import Instruction
# from ...nuc.object import OBJECT
# from ...nuc.object_class import OBJECT_CLASS
# from ...graph import graph
# from ...lib.random import random


def build(
    cls: Any,
    args: Optional[list[Any]] = None,
    object_: Optional[Any] = None,
    name: Optional[Any] = None,
) -> "DollarINSTANCE":
    """Build a DollarINSTANCE statement."""
    if args is None:
        args = []
    statement = DollarINSTANCE()
    statement.cls = cls
    statement.obj = object_
    statement.nme = name
    statement.args = args
    return statement


class DollarINSTANCE(Dollar):
    """Represents an instance creation statement."""

    def __init__(self) -> None:
        super().__init__()
        self.cls: Any = None
        self.args: list[Any] = []
        self.obj: Optional[Any] = None
        self.nme: Optional[Any] = None

    def before(self, scope: Any = None) -> None:
        """Prepare the instance creation."""
        # Imports moved here to avoid circular dependencies
        from ...lib.random import random

        if not self.obj and not self.nme:
            self.nme = random(16, True)

    def run(self, scope: Any) -> Any:
        """Execute the instance creation."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.object import OBJECT
        from ...nuc.object_class import OBJECT_CLASS
        from ...instruction import Instruction

        cls_name = f"${self.cls.name}" if hasattr(self.cls, "name") else f"${self.cls}"
        cls = DollarIdentifier(cls_name)
        name = DollarIdentifier(self.nme) if self.nme else None

        # Map each arg through proper handling based on its type
        args = []
        for arg in self.args:
            # Handle different expression types as needed
            if hasattr(arg, "type") and arg.type == "Identifier":
                args.append(DollarIdentifier(arg))
            else:
                # For other expression types, you might need different handling
                args.append(DollarIdentifier(arg))

        if not graph.retrieve(cls):
            raise ReferenceError(f"{cls} is not defined")

        if self.obj and name and name.to_string(scope) == "value":
            raise TypeError("Cannot use 'value' as a property")

        if self.obj:
            object_identifier = DollarIdentifier(self.obj)

            if not graph.retrieve(object_identifier):
                raise ReferenceError(f"{DollarIdentifier(self.obj)} is not defined")

            if isinstance(graph.retrieve(object_identifier.first), CLASS):
                statement = OBJECT_CLASS(f"{object_identifier}.{name}")
                setattr(statement, "class", graph.retrieve(cls))
                statement.name = name
                statement.object = graph.retrieve(object_identifier)
                return [
                    Instruction(scope, statement, True, True, False, False),
                    Instruction(scope, statement, False, False, True, True),
                ]
            else:
                statement = OBJECT(f"{object_identifier}.{name}")
                setattr(statement, "class", graph.retrieve(cls))
                statement.name = name
                statement.object = graph.retrieve(object_identifier)
                statement.arguments = args
                return statement

        statement = OBJECT(name)
        setattr(statement, "class", graph.retrieve(cls))
        statement.name = name
        statement.arguments = args
        return statement


# Export the function as dollar_instance for consistency
dollar_instance = build
