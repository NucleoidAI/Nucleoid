import ast
from ..lang.handlers.assignment_handler import assignment_handler
from ..lang.handlers.expression_handler import expression_handler

def process(parsed_tree):

    """ast.walk(node)
    Recursively yield all descendant nodes in the tree starting at node (including node itself), in no specified order.
    This is useful if you only want to modify nodes in place and don’t care about the context."""
    if isinstance(parsed_tree.body[0], ast.Expr):
        return expression_handler(parsed_tree.body[0].value)


    if isinstance(parsed_tree.body[0], ast.Assign):
        assignment_handler(parsed_tree.body[0])
