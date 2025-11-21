"""
Template literal AST node.
"""
from typing import Any, Optional, Union, Dict
import copy
from .__Node__ import __Node__


class __Template__(__Node__):
    """Template literal node."""
    
    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve template expressions in the given scope.
        
        Args:
            scope: The scope to resolve in
            
        Returns:
            Resolved template node
        """
        if scope:
            # Lazy import to avoid circular dependencies
            from .__Identifier__ import __Identifier__
            
            clone = copy.deepcopy(self.node)
            expressions = clone.get("expressions", [])
            
            clone["expressions"] = [
                __Identifier__(expression).resolve(scope)
                for expression in expressions
            ]
            
            return clone
        else:
            return self.node


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Template__:
    """
    Builder function for creating template instances.
    
    Args:
        node: The template node
        
    Returns:
        A __Template__ instance
    """
    return __Template__(node)


__template__ = build
