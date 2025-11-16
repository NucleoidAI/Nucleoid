"""
Base AST node class for Nucleoid language processing.
"""
from typing import Any, Dict, Type, Optional, Union, List
from abc import ABC


class __Node__(ABC):
    """Base class for all AST node types."""
    
    registry: Dict[str, Type["__Node__"]] = {}
    
    def __init__(self, node: Optional[Union[Dict[str, Any], str]] = None) -> None:
        """
        Initialize a node.
        
        Args:
            node: Either a dict representing an AST node, a string to parse, or None
        """
        self.iof: str = self.__class__.__name__
        
        if node is None:
            # Default to null literal
            self.node: Dict[str, Any] = {
                "type": "Literal",
                "value": None,
                "raw": "null"
            }
        elif isinstance(node, dict) and "type" in node:
            self.node = node
        elif isinstance(node, str):
            # Lazy import to avoid circular dependencies
            from ..lang.estree.parser import parse
            self.node = parse(node, False)
        else:
            self.node = node
    
    @classmethod
    def convert(cls, node: Optional[Union[Dict[str, Any], str]]) -> "__Node__":
        """
        Convert a node to the appropriate AST class instance.
        
        Args:
            node: The node to convert
            
        Returns:
            An instance of the appropriate AST class
        """
        if not node:
            return cls()
        
        # Parse string nodes
        if isinstance(node, str):
            from ..lang.estree.parser import parse
            node_obj = parse(node, False)
        else:
            node_obj = node
        
        node_type = node_obj.get("type")
        
        # Look up the appropriate class in the registry
        node_class = cls.registry.get(node_type)
        if node_class:
            return node_class(node_obj)
        
        return cls(node_obj)
    
    @classmethod
    def register(cls, node_type: str, node_class: Type["__Node__"]) -> None:
        """
        Register a node class for a specific type.
        
        Args:
            node_type: The AST node type string
            node_class: The class to use for this type
        """
        cls.registry[node_type] = node_class
    
    @property
    def type(self) -> str:
        """Get the node type."""
        return self.node.get("type", "")
    
    @property
    def first(self) -> Optional["__Node__"]:
        """Get the first node in a chain (for identifiers)."""
        return None
    
    @property
    def object(self) -> Optional["__Node__"]:
        """Get the object node (for member expressions)."""
        return None
    
    @property
    def last(self) -> Optional["__Node__"]:
        """Get the last node in a chain (for identifiers)."""
        return None
    
    def resolve(self, scope: Any) -> Dict[str, Any]:
        """
        Resolve the node in the given scope.
        
        Args:
            scope: The scope to resolve in
            
        Returns:
            The resolved node
        """
        return self.node
    
    def generate(self, scope: Any) -> str:
        """
        Generate code for this node.
        
        Args:
            scope: The scope to generate in
            
        Returns:
            Generated code string
        """
        from ..lang.estree.generator import ESTree
        resolved = self.resolve(scope)
        return ESTree.generate(resolved)
    
    def graph(self, scope: Any) -> List["__Node__"]:
        """
        Get graph of dependencies.
        
        Args:
            scope: The scope to analyze
            
        Returns:
            List of dependent nodes
        """
        return []
    
    def walk(self) -> List["__Node__"]:
        """
        Walk the AST tree.
        
        Returns:
            List of child nodes
        """
        return []
    
    def __str__(self) -> str:
        """String representation."""
        return self.generate(None)


def build(node: Optional[Union[Dict[str, Any], str]] = None) -> __Node__:
    """
    Builder function for creating node instances.
    
    Args:
        node: The node to wrap
        
    Returns:
        A __Node__ instance
    """
    return __Node__(node)


__node__ = build
