"""
Object expression AST node.
"""
from typing import Any, Optional, Union, Dict
import copy
from .__Node__ import __Node__


class __Object__(__Node__):
    """Object expression node."""
    
    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve object properties in the given scope.
        
        Args:
            scope: The scope to resolve in
            
        Returns:
            Resolved object node
        """
        if scope:
            # Lazy import to avoid circular dependencies
            from .__Identifier__ import __Identifier__
            
            cloned = copy.deepcopy(self.node)
            
            # Walk through properties and resolve identifiers
            properties = cloned.get("properties", [])
            for prop in properties:
                if prop.get("type") == "Property":
                    value = prop.get("value", {})
                    value_type = value.get("type", "")
                    
                    # Check if value is an identifier type
                    if value_type in __Identifier__.types:
                        identifier = __Identifier__(value)
                        prop["value"] = identifier.resolve(scope)
            
            return cloned
        else:
            return self.node


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Object__:
    """
    Builder function for creating object instances.
    
    Args:
        node: The object node
        
    Returns:
        An __Object__ instance
    """
    return __Object__(node)


__object__ = build
