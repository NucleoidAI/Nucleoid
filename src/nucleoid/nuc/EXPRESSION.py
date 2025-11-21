"""
EXPRESSION runtime node class.
"""
from typing import Any, List, Optional


class EXPRESSION:
    """Expression evaluation node."""

    def __init__(self, tokens: Any) -> None:
        """
        Initialize an EXPRESSION instance.

        Args:
            tokens: The expression tokens
        """
        self.iof: str = self.__class__.__name__
        self.tokens: Any = tokens

    def before(self, scope: Any) -> None:
        """
        Execute before the main run.

        Args:
            scope: The execution scope
        """
        # Lazy imports
        from ..lang.estree.parser import parse
        from ..lib.serialize import serialize
        from ..state import state_instance

        def process_node(node: Any) -> None:
            if (
                node.type == "MemberExpression"
                and hasattr(node, "last")
                and str(node.last) == "value"
            ):
                value = state_instance.expression(
                    scope, {"value": node.object.generate(scope)}
                )

                new_value = parse(serialize(value, "state"), False)

                # Update node in place
                for key in list(node.node.keys()):
                    del node.node[key]
                node.node.update(new_value)

        if hasattr(self.tokens, "map"):
            self.tokens.map(process_node)

    def run(self, scope: Any, force: bool = False) -> Optional[Any]:
        """
        Execute the expression.

        Args:
            scope: The execution scope
            force: Force evaluation even if undefined

        Returns:
            Evaluation result or None
        """
        # Lazy imports
        from ..lang.Evaluation import Evaluation
        from ..lang.estree.parser import parse
        from ..lib.serialize import serialize
        from ..state import state_instance
        from .. import graph

        # Process call expressions
        def process_call_node(node: Any) -> None:
            try:
                if node.type == "CallExpression":
                    func = state_instance.expression(
                        scope, {"value": node.function.generate(scope)}
                    )

                    if hasattr(func, "value") and func.value:
                        value = state_instance.expression(
                            scope, {"value": node.generate(scope)}
                        )
                        new_node = parse(serialize(value, "state"), False)

                        # Update node in place
                        for key in list(node.node.keys()):
                            del node.node[key]
                        node.node.update(new_node)

                    if hasattr(func, "write") and func.write:
                        self.tokens.wrt = True
            except Exception:
                # Deliberate empty catch block - preserving original behavior
                pass

        if hasattr(self.tokens, "map"):
            self.tokens.map(process_call_node)

        # Traverse and build expression
        def traverse_node(node: Any) -> str:
            evaluation = node.generate(scope)

            if scope.retrieve(node) or (
                node.type == "MemberExpression" and graph.retrieve(node.first)
            ):
                try:
                    test = state_instance.expression(scope, {"value": evaluation})
                    if test is None:
                        return "undefined"
                except Exception:
                    return "undefined"

            return evaluation

        expression: List[str] = []
        if hasattr(self.tokens, "traverse"):
            expression = self.tokens.traverse(traverse_node)

        if force or "undefined" not in expression:
            return Evaluation("".join(expression))

        return None

    def next(self) -> List[Any]:
        """
        Get next nodes in the graph.

        Returns:
            List of next nodes
        """
        from ..lang.ast import __Call__
        from .. import graph

        def process_ast(ast: Any) -> Optional[Any]:
            if isinstance(ast, __Call__):
                obj = ast.object
                if obj:
                    node = graph.retrieve(obj)
                    if node:
                        return list(node.next.values())
            return None

        if hasattr(self.tokens, "map"):
            return self.tokens.map(process_ast)
        return []

    def graph(self, scope: Any) -> Any:
        """
        Build dependency graph.

        Args:
            scope: The execution scope

        Returns:
            Graph result
        """
        from .. import graph
        from ..lang.ast import __Identifier__
        from ..lang.estree.estree import append
        from .NODE import NODE

        def graph_node(node: Any) -> Any:
            retrieved = graph.retrieve(node)

            if retrieved:
                return retrieved
            else:
                # Check if node is iterable
                if hasattr(node, "__iter__"):
                    for item in node:
                        if hasattr(item, "left") and hasattr(item, "right"):
                            left = item.left
                            right = item.right

                            test = graph.retrieve(left)

                            if (
                                test
                                and hasattr(test, "value")
                                and hasattr(test.value, "__class__")
                            ):
                                from .REFERENCE import REFERENCE

                                if isinstance(test.value, REFERENCE):
                                    link = __Identifier__(
                                        append(test.value.link.node, right.node)
                                    )
                                    return graph.retrieve(link)

                # Create temporary node
                temporary = NODE(node)
                graph.graph_instance[str(node)] = temporary
                return temporary

        if hasattr(self.tokens, "graph"):
            return self.tokens.graph(scope, graph_node)
        return None
