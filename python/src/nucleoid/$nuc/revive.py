from typing import Any, Union
from .__ import __
from .__ALIAS__ import __ALIAS__
from .__ASSIGNMENT__ import __ASSIGNMENT__
from .__BLOCK__ import __BLOCK__
from .__CALL__ import __CALL__
from .__CLASS__ import __CLASS__
from .__DELETE__ import __DELETE__
from .__EXPRESSION__ import __EXPRESSION__
from .__FOR__ import __FOR__
from .__FUNCTION__ import __FUNCTION__
from .__IF__ import __IF__
from .__INSTANCE__ import __INSTANCE__
from .__LET__ import __LET__
from .__PROPERTY__ import __PROPERTY__
from .__RETURN__ import __RETURN__
from .__THROW__ import __THROW__
from .__VARIABLE__ import __VARIABLE__

# Mapping of class names to their class objects
__CLASSES__ = {
    "__ALIAS__": __ALIAS__,
    "__ASSIGNMENT__": __ASSIGNMENT__,
    "__BLOCK__": __BLOCK__,
    "__CALL__": __CALL__,
    "__CLASS__": __CLASS__,
    "__DELETE__": __DELETE__,
    "__EXPRESSION__": __EXPRESSION__,
    "__FOR__": __FOR__,
    "__FUNCTION__": __FUNCTION__,
    "__IF__": __IF__,
    "__INSTANCE__": __INSTANCE__,
    "__LET__": __LET__,
    "__PROPERTY__": __PROPERTY__,
    "__RETURN__": __RETURN__,
    "__THROW__": __THROW__,
    "__VARIABLE__": __VARIABLE__,
}


def revive(statements: Union[list[__], __, dict, Any]) -> Any:
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
    if isinstance(statements, dict) or (hasattr(statements, '__dict__') and not isinstance(statements, __)):
        # Get the class name from iof attribute if present
        iof = None
        if isinstance(statements, dict):
            iof = statements.get('iof')
        else:
            iof = getattr(statements, 'iof', None)

        # Create new instance of the appropriate class
        if iof and iof in __CLASSES__:
            obj = __CLASSES__[iof]()
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
