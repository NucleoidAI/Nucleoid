from typing import Any, Union
from .dollar import Dollar
from .dollar_alias import DollarALIAS
from .dollar_assignment import DollarASSIGNMENT
from .dollar_block import DollarBLOCK
from .dollar_call import DollarCALL
from .dollar_class import DollarCLASS
from .dollar_delete import DollarDELETE
from .dollar_expression import DollarEXPRESSION
from .dollar_for import DollarFOR
from .dollar_function import DollarFUNCTION
from .dollar_if import DollarIF
from .dollar_instance import DollarINSTANCE
from .dollar_let import DollarLET
from .dollar_property import DollarPROPERTY
from .dollar_return import DollarRETURN
from .dollar_throw import DollarTHROW
from .dollar_variable import DollarVARIABLE
# from ...nuc.expression import EXPRESSION
# from ...expression import Expression

# Mapping of class names to their class objects
DOLLAR_CLASSES = {
    "DollarALIAS": DollarALIAS,
    "DollarASSIGNMENT": DollarASSIGNMENT,
    "DollarBLOCK": DollarBLOCK,
    "DollarCALL": DollarCALL,
    "DollarCLASS": DollarCLASS,
    "DollarDELETE": DollarDELETE,
    "DollarEXPRESSION": DollarEXPRESSION,
    "DollarFOR": DollarFOR,
    "DollarFUNCTION": DollarFUNCTION,
    "DollarIF": DollarIF,
    "DollarINSTANCE": DollarINSTANCE,
    "DollarLET": DollarLET,
    "DollarPROPERTY": DollarPROPERTY,
    "DollarRETURN": DollarRETURN,
    "DollarTHROW": DollarTHROW,
    "DollarVARIABLE": DollarVARIABLE,
}


def revive(statements: Union[list[Dollar], Dollar, dict, Any]) -> Any:
    """
    Revive statements from serialized form by reconstructing the objects.

    Args:
        statements: A list of statement objects, a single statement, or a dict/value

    Returns:
        The revived statement(s) with proper class instances
    """
    # Handle list of statements
    if isinstance(statements, list):
        return [revive(statement) for statement in statements]

    # Handle dict/object
    if isinstance(statements, dict) or (hasattr(statements, '__dict__') and not isinstance(statements, Dollar)):
        # Get the class name from iof attribute if present
        iof = None
        if isinstance(statements, dict):
            iof = statements.get('iof')
        else:
            iof = getattr(statements, 'iof', None)

        # Create new instance of the appropriate class
        if iof and iof in DOLLAR_CLASSES:
            obj = DOLLAR_CLASSES[iof]()
        else:
            obj = {}

        # Copy properties
        items = statements.items() if isinstance(statements, dict) else vars(statements).items()
        for key, property_value in items:
            # Recursively revive nested objects/arrays
            if isinstance(property_value, (list, dict)) or (
                hasattr(property_value, '__dict__')
                and not isinstance(property_value, (str, int, float, bool, type(None)))
                and property_value is not None
            ):
                if isinstance(obj, dict):
                    obj[key] = revive(property_value)
                else:
                    setattr(obj, key, revive(property_value))
            else:
                if isinstance(obj, dict):
                    obj[key] = property_value
                else:
                    setattr(obj, key, property_value)

        return obj

    # Return as-is if already a proper object or primitive
    return statements
