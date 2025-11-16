from typing import Any, Literal, Optional
from .__ import __


def build(
    kind: Optional[Literal["VAR", "LET", "CONST"]], left: Any, right: Any
) -> "__ASSIGNMENT__":
    """Build a __ASSIGNMENT__ statement."""
    statement = __ASSIGNMENT__()
    statement.knd = kind
    statement.lft = left
    statement.rgt = right
    return statement


class __ASSIGNMENT__(__):
    """Represents an assignment statement."""

    def __init__(self) -> None:
        super().__init__()
        self.knd: Optional[Literal["VAR", "LET", "CONST", "PROPERTY"]] = None
        self.lft: Any = None
        self.rgt: Any = None
        self._stmt: Optional[__] = None

    def before(self, scope: Any) -> None:
        """Prepare the assignment by determining the types and creating the appropriate statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from .__VARIABLE__ import __variable__
        from .__INSTANCE__ import __instance__
        from .__PROPERTY__ import __property__
        from .__LET__ import __let__

        name = __Identifier__(self.lft)
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

        if left_kind == "VAR" and right_kind == "EXPRESSION":
            self._stmt = __variable__(self.lft, self.rgt)
        elif left_kind == "VAR" and right_kind == "INSTANCE":
            rgt = self.rgt
            self._stmt = __instance__(
                rgt.callee, rgt.arguments if hasattr(rgt, "arguments") else [], None, self.lft
            )
        elif left_kind == "PROPERTY" and right_kind == "EXPRESSION":
            identifier = __Identifier__(self.lft)
            self._stmt = __property__(
                identifier.object.node, identifier.last.node, self.rgt
            )
        elif left_kind == "PROPERTY" and right_kind == "INSTANCE":
            identifier = __Identifier__(self.lft)
            rgt = self.rgt
            self._stmt = __instance__(
                rgt.callee,
                rgt.arguments if hasattr(rgt, "arguments") else [],
                identifier.object.node,
                identifier.last.node,
            )
        elif left_kind in ["LET", "CONST"]:
            self._stmt = __let__(self.lft, self.rgt, left_kind == "CONST", reassign)

        if self._stmt:
            self._stmt.asg = True

    def run(self, scope: Any) -> Any:
        """Execute the assignment statement."""
        from ...instruction import Instruction
        return Instruction(scope, self._stmt, None, True, None, None, None)
