from typing import Any, Union
from .__NUC__ import __NUC__


def build(tokens: Union[str, Any, "__Expression__"]) -> "__EXPRESSION__":
    """Build a __EXPRESSION__ statement."""
    from ..ast.__expression__ import __Expression__
    from ..estree.parser import parse

    if not isinstance(tokens, __Expression__):
        if isinstance(tokens, str):
            string = tokens
            tokens = __Expression__(parse(string, False))
        else:
            tokens = __Expression__(tokens)

    statement = __EXPRESSION__()
    statement.tkns = tokens
    return statement


class __EXPRESSION__(__NUC__):
    """Represents an expression statement."""

    def __init__(self) -> None:
        super().__init__()
        self.tkns: Any = None

    def run(self, scope: Any) -> Any:
        """Execute the expression statement."""
        from ..ast.__identifier__ import __Identifier__
        from ...nuc.expression import EXPRESSION
        from ...nuc.reference import REFERENCE
        from ...graph import graph
        from ...nuc.expression_instance import EXPRESSION_INSTANCE

        if hasattr(self.tkns, "node") and hasattr(self.tkns.node, "type"):
            node_type = self.tkns.node.type
            if node_type in __Identifier__.types:
                identifier = __Identifier__(self.tkns.node)
                if scope.retrieve(identifier):
                    return EXPRESSION(self.tkns)
                link = graph.retrieve(identifier)
                if link:
                    statement = REFERENCE(self.tkns)
                    statement.link = identifier
                    return statement
                else:
                    return EXPRESSION(self.tkns)

        __instance = getattr(scope, "$instance", None) if hasattr(scope, "$instance") else None
        if __instance:
            return EXPRESSION_INSTANCE(self.tkns)
        else:
            return EXPRESSION(self.tkns)


__expression__ = build
