"""Base NODE class for Nucleoid AI declarative constructs."""

from typing import Any, Dict, Optional
from .. import transaction
from ..graph import get_graph_manager


# Global sequence counter
_sequence = 0


class NODE:
    """Base class for all Nucleoid declarative constructs."""
    
    def __init__(self, key: Any):
        global _sequence
        self.key = str(key)
        self.next: Dict[str, Any] = {}
        self.previous: Dict[str, Any] = {}
        self.sequence = _sequence
        _sequence += 1
    
    def before(self, scope: Optional[Any] = None) -> None:
        """Execute before processing."""
        pass
    
    def run(self, scope: Optional[Any] = None) -> Any:
        """Execute the main logic."""
        pass
    
    def before_graph(self, scope: Optional[Any] = None) -> None:
        """Execute before graph processing."""
        pass
    
    def graph(self, scope: Optional[Any] = None) -> None:
        """Execute graph processing."""
        pass
    
    def after(self, scope: Optional[Any] = None) -> None:
        """Execute after processing."""
        pass
    
    @staticmethod
    def register(key: str, node: 'NODE') -> None:
        """Register a node in the graph."""
        graph_manager = get_graph_manager()
        # Use transaction to register changes
        transaction_manager = transaction.get_transaction_manager()
        if transaction_manager.is_active():
            transaction_manager.record_change("register", {"key": key, "node": node})
        
        # Add to graph
        graph_manager.add_node(key, type(node).__name__)
    
    @staticmethod
    def replace(source_key: str, target_node: 'NODE') -> None:
        """Replace a node in the graph."""
        graph_manager = get_graph_manager()
        transaction_manager = transaction.get_transaction_manager()
        
        source_node = graph_manager.get_node(source_key)
        if not source_node:
            return
        
        if transaction_manager.is_active():
            transaction_manager.record_change("replace", {
                "source_key": source_key,
                "target_node": target_node
            })
        
        # Update relationships
        for next_key in target_node.next:
            if next_key in source_node.relationships.get("next", []):
                target_node.next[next_key] = source_node.relationships["next"][next_key]
                del source_node.relationships["next"][next_key]
        
        # Update previous relationships
        for prev_key in source_node.relationships.get("previous", []):
            prev_node = graph_manager.get_node(prev_key)
            if prev_node and "next" in prev_node.relationships:
                if source_key in prev_node.relationships["next"]:
                    del prev_node.relationships["next"][source_key]
        
        # Replace the node
        graph_manager.remove_node(source_key)
        graph_manager.add_node(target_node.key, type(target_node).__name__)
    
    @staticmethod
    def direct(source_key: str, target_key: str, target_node: 'NODE') -> None:
        """Create a direct relationship between nodes."""
        graph_manager = get_graph_manager()
        transaction_manager = transaction.get_transaction_manager()
        
        source_node = graph_manager.get_node(source_key)
        if not source_node:
            return
        
        if transaction_manager.is_active():
            transaction_manager.record_change("direct", {
                "source_key": source_key,
                "target_key": target_key,
                "target_node": target_node
            })
        
        # Create relationships
        source_node.add_relationship("next", target_key)
        target_node.previous[source_key] = source_node
    
    def __str__(self) -> str:
        """String representation of the node."""
        return f"{self.__class__.__name__}({self.key})"
    
    def __repr__(self) -> str:
        """Detailed representation of the node."""
        return f"{self.__class__.__name__}(key={self.key}, sequence={self.sequence})"