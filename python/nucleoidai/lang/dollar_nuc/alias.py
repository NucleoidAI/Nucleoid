"""$ALIAS construct for Nucleoid AI."""

from typing import Any, Optional
from .base import DollarBase
from .expression import EXPRESSION, dollar_expression
from ..ast.identifier import Identifier
from ...nuc.alias_construct import ALIAS


def ALIAS(alias: Any, name: str, value: Any) -> 'DollarALIAS':
    """Build an ALIAS construct."""
    statement = DollarALIAS()
    statement.als = alias
    statement.nme = name  
    statement.val = value
    return statement


class DollarALIAS(DollarBase):
    """$ALIAS construct for creating aliases."""
    
    def __init__(self):
        super().__init__()
        self.als: Optional[Any] = None
        self.nme: Optional[str] = None
        self.val: Optional[Any] = None
    
    def before(self, scope: Optional[Any] = None) -> None:
        """Execute before processing."""
        expression = dollar_expression(self.val)
        self.val = expression.run(scope)
    
    def run(self, scope: Optional[Any] = None) -> Any:
        """Execute the alias creation."""
        name = Identifier(self.nme)
        statement = ALIAS()
        statement.alias = Identifier(self.als)
        statement.name = name
        statement.value = self.val
        return statement