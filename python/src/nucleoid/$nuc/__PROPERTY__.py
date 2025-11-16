from typing import Any
from .__ import __


def build(object_: Any, name: Any, value: Any) -> "__PROPERTY__":
    """Build a __PROPERTY__ statement."""
    statement = __PROPERTY__()
    statement.obj = object_
    statement.nme = name
    statement.val = value
    return statement


class __PROPERTY__(__):
    """Represents a property assignment statement."""

    def __init__(self) -> None:
        super().__init__()
        self.obj: Any = None
        self.nme: Any = None
        self.val: Any = None
        self._val: Any = None

    def before(self, scope: Any) -> None:
        """Prepare the property statement by evaluating the value."""
        from .__EXPRESSION__ import __expression__
        expression = __expression__(self.val)
        self._val = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the property statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.object_class import OBJECT_CLASS
        from ...nuc.function import FUNCTION
        from ...nuc.property_class import PROPERTY_CLASS
        from ...nuc.property import PROPERTY
        from ...instruction import Instruction

        object_id = __Identifier__(self.obj)
        name = __Identifier__(self.nme)

        if object_id.to_string(scope) == "this":
            object_id = scope.retrieve_object()

        if not graph.retrieve(object_id):
            raise ReferenceError(f"{object_id} is not defined")

        if (
            name.to_string(scope) == "value"
            and not isinstance(graph.get(object_id), FUNCTION)
        ):
            raise TypeError("Cannot use 'value' as a name")

        cls = graph.retrieve(object_id)
        if isinstance(cls, (CLASS, OBJECT_CLASS)):
            statement = PROPERTY_CLASS(f"{object_id}.{name}")
            setattr(statement, "class", cls)
            statement.object = graph.retrieve(object_id)
            statement.name = name
            statement.value = self.val
            return [
                Instruction(scope, statement, True, True, False, False),
                Instruction(scope, statement, False, False, True, True),
            ]

        statement = PROPERTY(f"{object_id}.{name}")
        statement.object = graph.retrieve(object_id)
        statement.name = name
        statement.value = self.val
        return Instruction(scope, statement, True, True, True, True)


__property__ = build
