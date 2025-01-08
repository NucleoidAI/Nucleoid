from ...nucleoid.state import variable_state
from ...nucleoid.graph import maingraph
import ast

def assignment_handler(node):
    # Extract the variable name from the target
    target = node.targets[0]
    if isinstance(target, ast.Name):
        var_name = target.id
        # Extract the value
        if isinstance(node.value, ast.Constant):
            var_value = node.value.value
        #THIS NEXT LINE IS FOR DEPENDENCY HANDLING
        #elif isinstance(node.value, ast.Name):
        #    var_value = variable_state.get(node.value.id, None)
        else:
            var_value = None  # Handle other types if necessary
            # Store the variable and its value in the dictionary
        variable_state[var_name] = var_value
        # Add the variable as a node in the graph
        maingraph.add_node(var_name)

