from typing import Any
from .__NUC__ import __NUC__


def build(key: Any) -> "__DELETE__":
    """Build a __DELETE__ statement."""
    statement = __DELETE__()
    statement.key = key
    return statement


class __DELETE__(__NUC__):
    """Represents a delete statement."""

    def __init__(self) -> None:
        super().__init__()
        self.key: Any = None

    def run(self, scope: Any) -> Any:
        """Execute the delete statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...graph import graph
        from ...nuc.variable import VARIABLE
        from ...nuc.delete_variable import DELETE_VARIABLE
        from ...nuc.delete_object import DELETE_OBJECT
        from ...nuc.object import OBJECT
        from ...nuc.delete import DELETE
        from .__EXPRESSION__ import __expression__
        from ...state import state

        identifier = __Identifier__(self.key)
        variable = graph.retrieve(identifier)

        if not variable:
            try:
                expression = __expression__(self.key)
                expr_result = expression.run(scope)
                item = expr_result.run(scope)
                result = state.expression(scope, {"value": item})
                id_value = result.get("id") if isinstance(result, dict) else getattr(result, "id", None)
                variable = graph.retrieve(id_value)
            except Exception:
                pass

        if isinstance(variable, VARIABLE):
            statement = DELETE_VARIABLE()
            statement.variable = variable
            return statement
        elif isinstance(variable, OBJECT):
            statement = DELETE_OBJECT()
            statement.variable = variable
            return statement
        else:
            statement = DELETE()
            statement.variable = variable
            return statement
