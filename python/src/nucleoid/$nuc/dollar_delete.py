from typing import Any
from .dollar import Dollar
# from ...nuc.delete import DELETE
# from ...nuc.variable import VARIABLE
# from ...graph import graph
# from ...nuc.delete_variable import DELETE_VARIABLE
# from ...nuc.delete_object import DELETE_OBJECT
# from ...nuc.object import OBJECT
# from ..ast.identifier import Identifier
# from .dollar_expression import dollar_expression
# from ...state import state


def build(key: Any) -> "DollarDELETE":
    """Build a DollarDELETE statement."""
    statement = DollarDELETE()
    statement.key = key
    return statement


class DollarDELETE(Dollar):
    """Represents a delete statement."""

    def __init__(self) -> None:
        super().__init__()
        self.key: Any = None

    def run(self, scope: Any) -> Any:
        """Execute the delete statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier
        from ...graph import graph
        from ...nuc.variable import VARIABLE
        from ...nuc.delete_variable import DELETE_VARIABLE
        from ...nuc.delete_object import DELETE_OBJECT
        from ...nuc.object import OBJECT
        from ...nuc.delete import DELETE
        from .dollar_expression import dollar_expression
        from ...state import state

        identifier = Identifier(self.key)
        variable = graph.retrieve(identifier)

        if not variable:
            try:
                expression = dollar_expression(self.key)
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
            # TODO Rename this for property
            statement = DELETE()
            statement.variable = variable
            return statement
