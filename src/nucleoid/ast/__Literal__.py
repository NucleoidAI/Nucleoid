"""
Literal AST node.
"""
from typing import Any, Optional, Union, Dict
from .__Node__ import __Node__


class __Literal__(__Node__):
    """Literal value node."""
    
    def generate(self, scope: Any = None) -> str:
        """
        Generate literal code.
        
        Args:
            scope: Unused for literals
            
        Returns:
            The raw literal string
        """
        return self.node.get("raw", "")


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Literal__:
    """
    Builder function for creating literal instances.
    
    Args:
        node: The literal node
        
    Returns:
        A __Literal__ instance
    """
    return __Literal__(node)


__literal__ = build
