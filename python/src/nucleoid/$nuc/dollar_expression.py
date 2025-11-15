from typing import Any, Union
from .dollar import Dollar
# from ...nuc.expression import EXPRESSION
# from ..ast.dollar_expression import DollarExpression
# from ..ast.identifier import Identifier as DollarIdentifier
# from ...nuc.reference import REFERENCE
# from ...graph import graph
# from ...nuc.expression_instance import EXPRESSION_INSTANCE
# from ..estree.parser import parse


def build(tokens: Union[str, Any, "DollarExpression"]) -> "DollarEXPRESSION":
    """Build a DollarEXPRESSION statement."""
    # Imports moved here to avoid circular dependencies
    from ..ast.dollar_expression import DollarExpression
    from ..estree.parser import parse

    if not isinstance(tokens, DollarExpression):
        if isinstance(tokens, str):
            string = tokens
            tokens = DollarExpression(parse(string, False))
        else:
            tokens = DollarExpression(tokens)

    statement = DollarEXPRESSION()
    statement.tkns = tokens
    return statement


class DollarEXPRESSION(Dollar):
    """Represents an expression statement."""

    def __init__(self) -> None:
        super().__init__()
        self.tkns: Any = None

    def run(self, scope: Any) -> Any:
        """Execute the expression statement."""
        # Imports moved here to avoid circular dependencies
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...nuc.expression import EXPRESSION
        from ...nuc.reference import REFERENCE
        from ...graph import graph
        from ...nuc.expression_instance import EXPRESSION_INSTANCE

        if hasattr(self.tkns, "node") and hasattr(self.tkns.node, "type"):
            node_type = self.tkns.node.type

            # Check if the node type is in the identifier types list
            if node_type in DollarIdentifier.types:
                identifier = DollarIdentifier(self.tkns.node)

                if scope.retrieve(identifier):
                    return EXPRESSION(self.tkns)

                link = graph.retrieve(identifier)

                if link:
                    statement = REFERENCE(self.tkns)
                    statement.link = identifier
                    return statement
                else:
                    return EXPRESSION(self.tkns)

        # Check for $instance in scope
        dollar_instance = getattr(scope, "$instance", None) if hasattr(scope, "$instance") else None

        if dollar_instance:
            return EXPRESSION_INSTANCE(self.tkns)
        else:
            return EXPRESSION(self.tkns)


# Export the function as dollar_expression for consistency
dollar_expression = build
