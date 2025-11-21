"""
AST conversion utilities.
"""
from typing import Union, Dict, Any
from .__Node__ import __Node__


class AST:
    """AST conversion utility class."""

    @staticmethod
    def convert(node: Union[Dict[str, Any], str]) -> __Node__:
        """
        Convert a node to the appropriate AST class.

        Args:
            node: The node to convert

        Returns:
            An instance of the appropriate AST class
        """
        # Lazy imports to avoid circular dependencies
        from .__Literal__ import __Literal__
        from .__Identifier__ import __Identifier__
        from .__Array__ import __Array__
        from .__New__ import __New__
        from .__Object__ import __Object__
        from .__Function__ import __Function__
        from .__Template__ import __Template__
        from .__Operator__ import __Operator__

        if isinstance(node, str):
            from ..lang.estree.parser import parse
            node = parse(node, False)

        node_type = node.get("type")

        if node_type == "Literal":
            return __Literal__(node)

        elif node_type in ["Identifier", "MemberExpression"]:
            return __Identifier__(node)

        elif node_type == "ArrayExpression":
            elements = node.get("elements", [])
            converted_elements = [AST.convert(el) for el in elements if el]
            return __Array__(converted_elements)

        elif node_type == "NewExpression":
            return __New__(node)

        elif node_type == "ObjectExpression":
            return __Object__(node)

        elif node_type in ["FunctionExpression", "ArrowFunctionExpression"]:
            return __Function__(node)

        # Uncomment when needed:
        # elif node_type == "CallExpression":
        #     return __Call__(node)

        elif node_type == "TemplateLiteral":
            return __Template__(node)

        else:
            # Default to operator for binary/logical/unary expressions
            return __Operator__(node)
