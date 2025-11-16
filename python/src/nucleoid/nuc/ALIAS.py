"""
ALIAS runtime node class.
"""
from typing import Any
from .VARIABLE import VARIABLE


class ALIAS(VARIABLE):
    """Alias variable node."""

    def __init__(self, value: Any) -> None:
        """
        Initialize an ALIAS instance.

        Args:
            value: The value node
        """
        super().__init__(value)
        self.alias: Any = None
        self.name: Any = None
        self.value: Any = None
