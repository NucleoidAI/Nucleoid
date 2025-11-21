"""
OBJECT_INSTANCE runtime node class.
"""
from typing import Any
from .NODE import NODE


class OBJECT_INSTANCE(NODE):
    """Object instance node."""

    def __init__(self, key: Any = None) -> None:
        """
        Initialize an OBJECT_INSTANCE.

        Args:
            key: The object instance key
        """
        super().__init__(key)
        self.class_ref: Any = None
        self.object: Any = None
