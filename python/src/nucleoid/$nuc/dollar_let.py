from typing import Any, Union
from .dollar import Dollar
# from ...graph import graph
# from ...nuc.class_ import CLASS
# from ...nuc.let import LET
# from ...nuc.let_class import LET_CLASS
# from ...nuc.let_object import LET_OBJECT
# from ...nuc.expression import EXPRESSION
# from ...nuc.reference import REFERENCE
# from .dollar_expression import dollar_expression
# from ..ast.identifier import Identifier as DollarIdentifier
# from .dollar_instance import dollar_instance
# from ...nuc.node import NODE


def build(
    name: Any, value: Any, constant: bool = False, reassign: bool = False
) -> "DollarLET":
    """Build a DollarLET statement."""
    statement = DollarLET()
    statement.nme = name
    statement.val = value
    statement.cst = constant
    statement.ras = reassign
    return statement


class DollarLET(Dollar):
    """Represents a let/const statement."""

    def __init__(self) -> None:
        super().__init__()
        self.nme: Any = None
        self.val: Any = None
        self.cst: bool = False
        self.ras: bool = False

    def before(self, scope: Any) -> None:
        """Prepare the let statement by evaluating the value."""
        # Imports moved here to avoid circular dependencies
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from .dollar_instance import dollar_instance
        from .dollar_expression import dollar_expression

        if (
            hasattr(self.val, "type")
            and self.val.type == "NewExpression"
            and hasattr(self.val, "callee")
            and hasattr(self.val.callee, "name")
            and isinstance(graph.retrieve(f"${self.val.callee.name}"), CLASS)
        ):
            args = self.val.arguments if hasattr(self.val, "arguments") else []
            self.val = dollar_instance(self.val.callee, args)
            self.val.before(scope)
        else:
            expression = dollar_expression(self.val)
            self.val = expression.run(scope)

    def run(self, scope: Any) -> Any:
        """Execute the let statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...nuc.expression import EXPRESSION
        from ...nuc.reference import REFERENCE
        from ...graph import graph
        from ...nuc.class_ import CLASS
        from ...nuc.let_class import LET_CLASS
        from ...nuc.let import LET
        from ...nuc.let_object import LET_OBJECT

        # TODO Rename this to `identifier`?
        name = DollarIdentifier(self.nme)

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
            # Find class in the expression tokens
            if hasattr(value, "tokens"):
                for node in value.tokens:
                    if hasattr(node, "walk"):
                        identifiers = []
                        walked = node.walk()
                        if isinstance(walked, list):
                            # Flatten nested lists
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
        elif hasattr(value, "type") and value.type == "DollarINSTANCE":
            obj = value.run(scope)

            statement = LET_OBJECT()
            statement.name = name
            statement.object = obj
            statement.constant = self.cst

            return [obj, statement]


# Export the function as dollar_let for consistency
dollar_let = build
