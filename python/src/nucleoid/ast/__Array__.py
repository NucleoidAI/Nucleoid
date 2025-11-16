"""
Array AST node.
"""
from typing import Any, List, Optional, Union, Dict
from .__Node__ import __Node__


class __Array__(__Node__):
    """Array expression node."""
    
    def __init__(self, elements: Union[List["__Node__"], Dict[str, Any], str, None]) -> None:
        """
        Initialize an array node.
        
        Args:
            elements: List of element nodes or an AST node
        """
        if isinstance(elements, list):
            super().__init__(None)
            self.elements = elements
        else:
            super().__init__(elements)
            self.elements = elements if isinstance(elements, list) else []
    
    def generate(self, scope: Any) -> str:
        """
        Generate array literal code.
        
        Args:
            scope: The scope to generate in
            
        Returns:
            Generated array literal string
        """
        element_strs = [el.generate(scope) for el in self.elements]
        return f"[{','.join(element_strs)}]"


def build(elements: Union[List["__Node__"], Dict[str, Any], str, None] = None) -> __Array__:
    """
    Builder function for creating array instances.
    
    Args:
        elements: The array elements
        
    Returns:
        An __Array__ instance
    """
    return __Array__(elements)


__array__ = build
