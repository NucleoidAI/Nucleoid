"""
RETURN runtime node class.
"""
from typing import Any
from .NODE import NODE


class RETURN(NODE):
    """Return statement node."""

    def __init__(self, statement: Any = None) -> None:
        """
        Initialize a RETURN instance.

        Args:
            statement: The return statement
        """
        super().__init__()
        self.statement: Any = statement
