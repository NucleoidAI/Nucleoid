"""
BREAK runtime node class.
"""
from typing import Any


class BREAK:
    """Break statement node."""

    def __init__(self, block: Any = None) -> None:
        """
        Initialize a BREAK instance.

        Args:
            block: The block to break from
        """
        self.block: Any = block
