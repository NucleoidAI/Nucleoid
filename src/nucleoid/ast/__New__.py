"""
New expression AST node.
"""
from typing import Optional, Union, Dict, Any
from .__Node__ import __Node__


class __New__(__Node__):
    """New expression node."""
    pass


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __New__:
    """
    Builder function for creating new expression instances.
    
    Args:
        node: The new expression node
        
    Returns:
        A __New__ instance
    """
    return __New__(node)


__new__ = build
