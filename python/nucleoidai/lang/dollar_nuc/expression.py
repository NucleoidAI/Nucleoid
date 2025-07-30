"""$EXPRESSION construct for Nucleoid AI."""

from typing import Any, Optional
from .base import DollarBase


def EXPRESSION(expression: Any) -> 'DollarEXPRESSION':
    """Build an EXPRESSION construct."""
    statement = DollarEXPRESSION()
    statement.expression = expression
    return statement


def dollar_expression(value: Any) -> 'DollarEXPRESSION':
    """Create a dollar expression from a value."""
    return EXPRESSION(value)


class DollarEXPRESSION(DollarBase):
    """$EXPRESSION construct for handling expressions."""
    
    def __init__(self):
        super().__init__()
        self.expression: Optional[Any] = None
    
    def run(self, scope: Optional[Any] = None) -> Any:
        """Execute the expression."""
        return self.expression
    
    def __str__(self) -> str:
        return f"$EXPRESSION({self.expression})"