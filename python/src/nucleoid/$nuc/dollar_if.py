from typing import Any, Optional
from .dollar import Dollar
# from ...graph import graph
# from ...nuc.if_ import IF
# from ...nuc.class_ import CLASS
# from ...nuc.if_class import IF_CLASS
# from ...instruction import Instruction
# from ..ast.dollar_expression import DollarExpression
# from .dollar_expression import dollar_expression
# from ..ast.identifier import Identifier as DollarIdentifier
# from .dollar_block import DollarBLOCK
# from ...nuc.node import NODE
# from ...nuc.expression import EXPRESSION


def build(
    condition: Any, true_statement: "DollarBLOCK", false_statement: Optional["DollarBLOCK"]
) -> "DollarIF":
    """Build a DollarIF statement."""
    statement = DollarIF()
    statement.con = condition
    statement.tru = true_statement  # truthy
    statement.fls = false_statement  # falsy
    return statement


class DollarIF(Dollar):
    """Represents an if statement."""

    def __init__(self) -> None:
        super().__init__()
        self.con: Any = None
        self.tru: Optional["DollarBLOCK"] = None
        self.fls: Optional["DollarBLOCK"] = None

    def before(self, scope: Any) -> None:
        """Prepare the if statement by evaluating the condition."""
        # Imports moved here to avoid circular dependencies
        from ..ast.dollar_expression import DollarExpression
        from .dollar_expression import dollar_expression

        condition = DollarExpression(self.con)
        expression = dollar_expression(condition)
        self.con = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the if statement."""
        # Imports moved here to avoid circular dependencies
        from ...nuc.expression import EXPRESSION
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.if_class import IF_CLASS
        from ...nuc.if_ import IF
        from ...instruction import Instruction

        con = self.con

        # Look up first expression for deciding class declaration
        declarations = con.graph(scope) if hasattr(con, "graph") else None

        if declarations and len(declarations) > 0:
            declaration = declarations[0]
            key = getattr(declaration, "key", None)
            if key:
                identifier = DollarIdentifier(key)
                cls = graph.retrieve(identifier.first)

                if isinstance(cls, CLASS):
                    statement = IF_CLASS(f"if({con.tokens})")
                    setattr(statement, "class", cls)
                    statement.condition = con
                    statement.true = self.tru

                    if self.fls:
                        setattr(statement, "false", self.fls)

                    return [
                        Instruction(scope, statement, True, True, False, False),
                        Instruction(scope, statement, False, False, True, True),
                    ]

        statement = IF(f"if({con.tokens})")
        statement.condition = con
        statement.true = self.tru
        setattr(statement, "false", self.fls)

        return [
            Instruction(scope, statement, True, True, False, False),
            Instruction(scope, statement, False, False, True, True),
        ]
