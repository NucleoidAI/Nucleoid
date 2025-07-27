"""
Deep utility functions for object manipulation.

This module provides deep comparison, merging, and equality functions
for working with nested data structures.
"""

from typing import Any


def deep_equal(obj1: Any, obj2: Any) -> bool:
    """
    Perform deep equality comparison between two objects.
    
    Args:
        obj1: First object to compare
        obj2: Second object to compare
        
    Returns:
        True if objects are deeply equal, False otherwise
    """
    if obj1 is obj2:
        return True

    if (
        not isinstance(obj1, (dict, list)) or
        not isinstance(obj2, (dict, list)) or
        type(obj1) != type(obj2)
    ):
        return obj1 == obj2

    if isinstance(obj1, list):
        if len(obj1) != len(obj2):
            return False
        return all(deep_equal(a, b) for a, b in zip(obj1, obj2, strict=False))

    if isinstance(obj1, dict):
        if len(obj1) != len(obj2):
            return False

        for key in obj1:
            if key not in obj2 or not deep_equal(obj1[key], obj2[key]):
                return False

        return True

    return False


def deep_merge(target: dict[str, Any], source: dict[str, Any]) -> dict[str, Any]:
    """
    Deeply merge two dictionaries.
    
    Args:
        target: Target dictionary to merge into
        source: Source dictionary to merge from
        
    Returns:
        New dictionary with merged values
    """
    output = target.copy()

    for key, value in source.items():
        if (
            key in output and
            _is_dict(output[key]) and
            _is_dict(value)
        ):
            output[key] = deep_merge(output[key], value)
        else:
            output[key] = value

    return output


def equal(actual: Any, expected: Any) -> bool:
    """
    Simple equality comparison.
    
    Args:
        actual: Actual value
        expected: Expected value
        
    Returns:
        True if values are equal
    """
    return actual == expected


def _is_dict(item: Any) -> bool:
    """
    Check if an item is a dictionary.
    
    Args:
        item: Item to check
        
    Returns:
        True if item is a dictionary
    """
    return isinstance(item, dict)
