from typing import Any, Union
from .__NUC__ import __NUC__


def build(
    name: Any, value: Any, constant: bool = False, reassign: bool = False
) -> "__LET__":
    """Build a __LET__ statement."""
    statement = __LET__()
    statement.nme = name
    statement.val = value
    statement.cst = constant
    statement.ras = reassign
    return statement


class __LET__(__NUC__):
    """Represents a let/const statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.val: Any = None
        self.cst: bool = False
        self.ras: bool = False

    def before(self, scope: Any) -> None:
        """Prepare the let statement by evaluating the value."""
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from .__INSTANCE__ import __instance__
        from .__EXPRESSION__ import __expression__

        if (
            hasattr(self.val, "type")
            and self.val.type == "NewExpression"
            and hasattr(self.val, "callee")
            and hasattr(self.val.callee, "name")
            and isinstance(graph.retrieve(f"${self.val.callee.name}"), CLASS)
        ):
            args = self.val.arguments if hasattr(self.val, "arguments") else []
            self.val = __instance__(self.val.callee, args)
            self.val.before(scope)
        else:
            expression = __expression__(self.val)
            self.val = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the let statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.expression import EXPRESSION
        from ...nuc.reference import REFERENCE
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.let_class import LET_CLASS
        from ...nuc.let import LET
        from ...nuc.let_object import LET_OBJECT

        name = __Identifier__(self.nme)

        if name.type == "MemberExpression" and not scope.retrieve(name.object, True):
            raise ReferenceError(f"{name.object} is not defined")

        if (
            name.type == "MemberExpression"
            and hasattr(name, "last")
            and name.last
            and name.last.to_string(scope) == "value"
        ):
            raise TypeError("Cannot use 'value' in local")

        value = self.val

        if isinstance(value, (EXPRESSION, REFERENCE)):
            cls = None
            if hasattr(value, "tokens"):
                for node in value.tokens:
                    if hasattr(node, "walk"):
                        identifiers = []
                        walked = node.walk()
                        if isinstance(walked, list):
                            stack = [walked]
                            while stack:
                                current = stack.pop()
                                if isinstance(current, list):
                                    stack.extend(current)
                                else:
                                    identifiers.append(current)
                        else:
                            identifiers = [walked]

                        for identifier in identifiers:
                            if hasattr(identifier, "first"):
                                cls_candidate = graph.retrieve(identifier.first)
                                if isinstance(cls_candidate, CLASS):
                                    cls = cls_candidate
                                    break
                        if cls:
                            break

            if cls:
                statement = LET_CLASS()
                setattr(statement, "class", cls)
                statement.name = name
                statement.value = value
                statement.constant = self.cst
                return statement

            statement = LET()
            statement.name = name
            statement.value = value
            statement.constant = self.cst
            statement.reassign = self.ras
            return statement
        elif hasattr(value, "type") and value.type == "__INSTANCE__":
            obj = value.run(scope)
            statement = LET_OBJECT()
            statement.name = name
            statement.object = obj
            statement.constant = self.cst
            return [obj, statement]


__let__ = build
