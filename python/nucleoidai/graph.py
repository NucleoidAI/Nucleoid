"""Graph processing module for Nucleoid AI."""

from typing import Dict, Any, Optional, List, Union


class GraphNode:
    """Represents a node in the knowledge graph."""
    
    def __init__(self, key: str, node_type: str = "default"):
        self.key = key
        self.type = node_type
        self.properties: Dict[str, Any] = {}
        self.relationships: Dict[str, List[str]] = {}
    
    def add_property(self, name: str, value: Any) -> None:
        """Add a property to the node."""
        self.properties[name] = value
    
    def add_relationship(self, relation_type: str, target_key: str) -> None:
        """Add a relationship to another node."""
        if relation_type not in self.relationships:
            self.relationships[relation_type] = []
        self.relationships[relation_type].append(target_key)
    
    def __str__(self) -> str:
        return f"GraphNode({self.key}, {self.type})"


class GraphManager:
    """Manages the knowledge graph for Nucleoid AI."""
    
    def __init__(self):
        self._graph: Dict[str, GraphNode] = {}
    
    def add_node(self, key: str, node_type: str = "default") -> GraphNode:
        """Add a node to the graph."""
        node = GraphNode(key, node_type)
        self._graph[key] = node
        return node
    
    def get_node(self, key: str) -> Optional[GraphNode]:
        """Get a node by key."""
        return self._graph.get(key)
    
    def remove_node(self, key: str) -> bool:
        """Remove a node from the graph."""
        if key in self._graph:
            del self._graph[key]
            
            # Remove relationships pointing to this node
            for node in self._graph.values():
                for relation_type, targets in node.relationships.items():
                    if key in targets:
                        targets.remove(key)
            
            return True
        return False
    
    def add_relationship(self, from_key: str, relation_type: str, to_key: str) -> bool:
        """Add a relationship between two nodes."""
        from_node = self.get_node(from_key)
        to_node = self.get_node(to_key)
        
        if from_node and to_node:
            from_node.add_relationship(relation_type, to_key)
            return True
        return False
    
    def get_graph(self) -> Dict[str, GraphNode]:
        """Get the entire graph."""
        return self._graph.copy()
    
    def clear(self) -> None:
        """Clear the entire graph."""
        self._graph = {}
    
    def size(self) -> int:
        """Get the number of nodes in the graph."""
        return len(self._graph)
    
    def find_nodes_by_type(self, node_type: str) -> List[GraphNode]:
        """Find all nodes of a specific type."""
        return [node for node in self._graph.values() if node.type == node_type]
    
    def get_neighbors(self, key: str, relation_type: Optional[str] = None) -> List[GraphNode]:
        """Get neighboring nodes."""
        node = self.get_node(key)
        if not node:
            return []
        
        neighbors = []
        for rel_type, targets in node.relationships.items():
            if relation_type is None or rel_type == relation_type:
                for target_key in targets:
                    target_node = self.get_node(target_key)
                    if target_node:
                        neighbors.append(target_node)
        
        return neighbors


# Global graph manager instance
_graph_manager = GraphManager()

def get_graph_manager() -> GraphManager:
    """Get the global graph manager instance."""
    return _graph_manager