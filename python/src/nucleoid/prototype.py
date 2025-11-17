"""
Prototype module for extending built-in types.

This module adds custom attributes to built-in Python types for Nucleoid runtime.
"""
import datetime
from typing import Any


def init() -> None:
    """
    Initialize prototype extensions.

    Note: Python doesn't allow direct modification of built-in types like JavaScript does.
    This function serves as a placeholder for any prototype-related initialization
    that might be needed in the Python implementation.
    """
    # In JavaScript, we can add properties to built-ins:
    # Date.now.value = true
    # Array.prototype.push.write = true

    # In Python, we can't modify built-ins directly, but we can:
    # 1. Create wrapper classes
    # 2. Use monkey patching (not recommended for built-ins)
    # 3. Track these properties separately in the runtime

    # For now, this is a placeholder that documents the behavior
    # without actually modifying built-ins
    pass


# Custom attributes that would be on built-ins in JavaScript version
BUILTIN_ATTRIBUTES = {
    'datetime.datetime.now': {'value': True},
    'list.append': {'write': True},
}


def has_attribute(obj: Any, attr: str, key: str) -> bool:
    """
    Check if a built-in function has a custom attribute.

    Args:
        obj: Object type or instance
        attr: Attribute name
        key: Attribute key

    Returns:
        True if attribute exists
    """
    type_name = f"{obj.__module__}.{obj.__class__.__name__}" if hasattr(obj, '__class__') else str(obj)
    full_key = f"{type_name}.{attr}"

    if full_key in BUILTIN_ATTRIBUTES:
        return key in BUILTIN_ATTRIBUTES[full_key]

    return False


def get_attribute(obj: Any, attr: str, key: str) -> Any:
    """
    Get a custom attribute from a built-in function.

    Args:
        obj: Object type or instance
        attr: Attribute name
        key: Attribute key

    Returns:
        Attribute value or None
    """
    type_name = f"{obj.__module__}.{obj.__class__.__name__}" if hasattr(obj, '__class__') else str(obj)
    full_key = f"{type_name}.{attr}"

    if full_key in BUILTIN_ATTRIBUTES:
        return BUILTIN_ATTRIBUTES[full_key].get(key)

    return None
