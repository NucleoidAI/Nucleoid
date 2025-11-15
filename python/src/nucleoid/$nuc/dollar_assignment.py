from typing import Any, Literal, Optional
from .dollar import Dollar
# from .dollar_property import dollar_property
# from .dollar_variable import dollar_variable
# from .dollar_let import dollar_let
# from .dollar_instance import dollar_instance
# from ..ast.identifier import Identifier as DollarIdentifier
# from ...graph import graph
# from ...nuc.class_ import CLASS
# from ...instruction import Instruction


def build(
    kind: Optional[Literal["VAR", "LET", "CONST"]], left: Any, right: Any
) -> "DollarASSIGNMENT":
    """Build a DollarASSIGNMENT statement."""
    statement = DollarASSIGNMENT()
    statement.knd = kind
    statement.lft = left
    statement.rgt = right
    return statement


class DollarASSIGNMENT(Dollar):
    """Represents an assignment statement."""

    def __init__(self) -> None:
        super().__init__()
        self.knd: Optional[Literal["VAR", "LET", "CONST", "PROPERTY"]] = None
        self.lft: Any = None
        self.rgt: Any = None
        self.dollar: Optional[Dollar] = None

    def before(self, scope: Any) -> None:
        """Prepare the assignment by determining the types and creating the appropriate statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from .dollar_variable import dollar_variable
        from .dollar_instance import dollar_instance
        from .dollar_property import dollar_property
        from .dollar_let import dollar_let

        name = DollarIdentifier(self.lft)

        left_kind = self.knd
        reassign = not self.knd

        if not left_kind:
            if scope.retrieve(name):
                left_kind = "LET"
            else:
                if hasattr(self.lft, "type") and self.lft.type == "Identifier":
                    left_kind = "VAR"
                else:
                    left_kind = "PROPERTY"

        right_kind = None

        if not self.rgt:
            raise SyntaxError("Missing definition")

        if hasattr(self.rgt, "type") and self.rgt.type == "NewExpression":
            if (
                hasattr(self.rgt, "callee")
                and hasattr(self.rgt.callee, "name")
                and isinstance(graph.retrieve(f"${self.rgt.callee.name}"), CLASS)
            ):
                right_kind = "INSTANCE"
            else:
                right_kind = "EXPRESSION"
        else:
            right_kind = "EXPRESSION"

        # Match TypeScript switch-case logic
        if left_kind == "VAR" and right_kind == "EXPRESSION":
            self.dollar = dollar_variable(self.lft, self.rgt)
        elif left_kind == "VAR" and right_kind == "INSTANCE":
            rgt = self.rgt
            self.dollar = dollar_instance(
                rgt.callee, rgt.arguments if hasattr(rgt, "arguments") else [], None, self.lft
            )
        elif left_kind == "PROPERTY" and right_kind == "EXPRESSION":
            identifier = DollarIdentifier(self.lft)
            self.dollar = dollar_property(
                identifier.object.node, identifier.last.node, self.rgt
            )
        elif left_kind == "PROPERTY" and right_kind == "INSTANCE":
            identifier = DollarIdentifier(self.lft)
            rgt = self.rgt
            self.dollar = dollar_instance(
                rgt.callee,
                rgt.arguments if hasattr(rgt, "arguments") else [],
                identifier.object.node,
                identifier.last.node,
            )
        elif left_kind in ["LET", "CONST"]:
            self.dollar = dollar_let(self.lft, self.rgt, left_kind == "CONST", reassign)

        if self.dollar:
            self.dollar.asg = True  # assigned

    def run(self, scope: Any) -> Any:
        """Execute the assignment statement."""
        from ...instruction import Instruction

        return Instruction(scope, self.dollar, None, True, None, None, None)
