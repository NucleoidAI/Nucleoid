"""
Deep comparison and merge utilities for objects.

Note: This module is deprecated in favor of more robust libraries,
but maintained for compatibility.
"""

from typing import Any


def deep_equal(obj1: object, obj2: object) -> bool:
    """
    Recursively compare two objects for deep equality.

    Args:
        obj1: First object to compare
        obj2: Second object to compare

    Returns:
        True if objects are deeply equal, False otherwise
    """
    # Same object reference
    if obj1 is obj2:
        return True

    # Check if both are None or if types differ
    if obj1 is None or obj2 is None:
        return False

    if type(obj1) != type(obj2):
        return False

    # For non-dict/non-list objects, use regular equality
    if not isinstance(obj1, (dict, list)):
        return obj1 == obj2

    # Handle lists
    if isinstance(obj1, list) and isinstance(obj2, list):
        if len(obj1) != len(obj2):
            return False
        return all(deep_equal(a, b) for a, b in zip(obj1, obj2))

    # Handle dictionaries
    if isinstance(obj1, dict) and isinstance(obj2, dict):
        keys1 = list(obj1.keys())
        keys2 = list(obj2.keys())

        if len(keys1) != len(keys2):
            return False

        for key in keys1:
            if key not in keys2:
                return False
            if not deep_equal(obj1[key], obj2[key]):
                return False

        return True

    # Default to regular equality
    return obj1 == obj2


def equal(actual: Any, expected: Any) -> bool:
    """
    Simple equality comparison.

    Args:
        actual: Actual value
        expected: Expected value

    Returns:
        True if values are equal, False otherwise
    """
    return actual == expected


def deep_merge(target: dict, source: dict) -> dict:
    """
    Recursively merge source dictionary into target dictionary.

    Args:
        target: Target dictionary to merge into
        source: Source dictionary to merge from

    Returns:
        A new dictionary with merged values
    """
    output = {**target}

    for key in source:
        if key in source:
            if _is_object(source[key]) and key in target and _is_object(target[key]):
                output[key] = deep_merge(target[key], source[key])
            else:
                output[key] = source[key]

    return output


def _is_object(item: Any) -> bool:
    """
    Check if an item is a plain dictionary object.

    Args:
        item: Item to check

    Returns:
        True if item is a dict (not a list or None), False otherwise
    """
    return bool(item and isinstance(item, dict) and not isinstance(item, list))
