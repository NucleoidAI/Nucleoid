"""
Serializes Python values to a string representation.
Handles circular references by using a reference system.
"""

import json
import re
from datetime import datetime
from typing import Any


def serialize(
    value: Any, source: str, refs: dict[int, str] | None = None
) -> str:
    """
    Serializes Python values to a string representation.
    Handles circular references by using a reference system.

    Args:
        value: The value to serialize
        source: Source identifier for reference tracking
        refs: Internal parameter used for tracking object references

    Returns:
        A string representation of the value
    """
    if refs is None:
        refs = {}

    # Handle None
    if value is None:
        return "null"

    # Get the type of the value
    value_type = type(value)

    # Handle strings
    if isinstance(value, str):
        return json.dumps(value)

    # Handle numbers (int and float)
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return str(value)

    # Handle booleans (must come after numbers since bool is a subclass of int)
    if isinstance(value, bool):
        return "true" if value else "false"

    # Handle functions
    if callable(value) and not isinstance(value, type):
        # For functions, return their source code representation
        import inspect
        try:
            source_code = inspect.getsource(value).strip()
            # Clean up indentation
            lines = source_code.split('\n')
            # Remove common leading whitespace
            if lines:
                min_indent = min(
                    len(line) - len(line.lstrip())
                    for line in lines if line.strip()
                )
                source_code = '\n'.join(
                    line[min_indent:] if len(line) > min_indent else line
                    for line in lines
                )
            return source_code
        except (OSError, TypeError):
            # If we can't get the source, return a generic representation
            return f"function {value.__name__}"

    # Handle objects (must check for circular references)
    if isinstance(value, (dict, list, set, datetime)) or hasattr(value, '__dict__'):
        # Check for circular references using object id
        obj_id = id(value)
        if obj_id in refs:
            return f"{{$ref:{{id:'{refs[obj_id]}',source:'{source}'}}}}"

        # Handle lists
        if isinstance(value, list):
            return json.dumps(value)

        # Handle datetime
        if isinstance(value, datetime):
            # Convert to milliseconds timestamp
            timestamp = int(value.timestamp() * 1000)
            return f"new Date({timestamp})"

        # Handle regex patterns
        if isinstance(value, re.Pattern):
            return value.pattern if hasattr(value, 'pattern') else str(value)

        # Handle sets
        if isinstance(value, set):
            values = [serialize(v, source, refs) for v in value]
            return f"new Set([{','.join(values)}])"

        # Handle dictionaries (Map-like)
        if isinstance(value, dict):
            # Check if this is a regular dict or should be treated as a Map
            # For now, treat all dicts as objects unless they need special handling

            # Store the reference ID for this object if it has an 'id' field
            if 'id' in value:
                refs[obj_id] = str(value['id'])

            # Check if it's a plain dictionary
            entries = [f"{k}:{serialize(v, source, refs)}" for k, v in value.items()]
            return f"{{{','.join(entries)}}}"

    # Default: convert to string
    return str(value)
