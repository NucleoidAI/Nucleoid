from typing import Any, Optional
from .__NUC__ import __NUC__


def build(
    condition: Any, true_statement: "__BLOCK__", false_statement: Optional["__BLOCK__"]
) -> "__IF__":
    """Build a __IF__ statement."""
    statement = __IF__()
    statement.con = condition
    statement.tru = true_statement
    statement.fls = false_statement
    return statement


class __IF__(__NUC__):
    """Represents an if statement."""

    def __init__(self) -> None:
        super().__init__()
        self.con: Any = None
        self.tru: Optional["__BLOCK__"] = None
        self.fls: Optional["__BLOCK__"] = None

    def before(self, scope: Any) -> None:
        """Prepare the if statement by evaluating the condition."""
        from ..ast.__expression__ import __Expression__
        from .__EXPRESSION__ import __expression__

        condition = __Expression__(self.con)
        expression = __expression__(condition)
        self.con = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the if statement."""
        from ...nuc.expression import EXPRESSION
        from ..ast.__identifier__ import __Identifier__
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.if_class import IF_CLASS
        from ...nuc.if_ import IF
        from ...instruction import Instruction

        con = self.con
        declarations = con.graph(scope) if hasattr(con, "graph") else None

        if declarations and len(declarations) > 0:
            declaration = declarations[0]
            key = getattr(declaration, "key", None)
            if key:
                identifier = __Identifier__(key)
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
