import ast
from ...nucleoid.state import variable_state

def expression_handler(node):
    """
    Evaluates an AST node and returns its value based on the variable_state dictionary.

    Args:
        node (ast.Node): The AST node to evaluate.
        variable_state (dict): A dictionary containing variable names and their values.

    Returns:
        The evaluated value of the node.

    Raises:
        NameError: If a variable is not defined in variable_state.
        NotImplementedError: If the node type or operation is not supported.
    """
    if isinstance(node, ast.Name):
        if node.id in variable_state:
            return variable_state[node.id]
        else:
            raise NameError(f"Variable {node.id} is not defined")
    elif isinstance(node, ast.Constant):
        return node.value
    elif isinstance(node, ast.BinOp):
        left = expression_handler(node.left)
        right = expression_handler(node.right)
        if isinstance(node.op, ast.Add):
            return left + right
        elif isinstance(node.op, ast.Sub):
            return left - right
        elif isinstance(node.op, ast.Mult):
            return left * right
        elif isinstance(node.op, ast.Div):
            return left / right
        # Add more operators as needed
        else:
            raise NotImplementedError(f"Operator {node.op} not supported")
    elif isinstance(node, ast.Compare):
        left = expression_handler(node.left)
        right = expression_handler(node.comparators[0])
        if isinstance(node.ops[0], ast.Eq):
            return left == right
        elif isinstance(node.ops[0], ast.NotEq):
            return left != right
        elif isinstance(node.ops[0], ast.Lt):
            return left < right
        elif isinstance(node.ops[0], ast.LtE):
            return left <= right
        elif isinstance(node.ops[0], ast.Gt):
            return left > right
        elif isinstance(node.ops[0], ast.GtE):
            return left >= right
        # Add more comparison operators as needed
        else:
            raise NotImplementedError(f"Comparison operator {node.ops[0]} not supported")
    else:
        raise NotImplementedError(f"Node type {type(node)} not supported")
