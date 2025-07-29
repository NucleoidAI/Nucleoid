"""Tests for Nucleoid AI graph functionality."""

import pytest
from nucleoidai.graph import GraphManager, GraphNode


class TestGraphNode:
    """Test graph node functionality."""
    
    def test_create_node(self):
        """Test creating a graph node."""
        node = GraphNode("test_key", "test_type")
        assert node.key == "test_key"
        assert node.type == "test_type"
        assert isinstance(node.properties, dict)
        assert isinstance(node.relationships, dict)
    
    def test_add_property(self):
        """Test adding properties to a node."""
        node = GraphNode("test_key")
        node.add_property("name", "test_value")
        assert node.properties["name"] == "test_value"
    
    def test_add_relationship(self):
        """Test adding relationships to a node."""
        node = GraphNode("test_key")
        node.add_relationship("connected_to", "other_key")
        assert "connected_to" in node.relationships
        assert "other_key" in node.relationships["connected_to"]


class TestGraphManager:
    """Test graph manager functionality."""
    
    def setup_method(self):
        """Set up test environment."""
        self.graph_manager = GraphManager()
    
    def test_add_and_get_node(self):
        """Test adding and retrieving nodes."""
        node = self.graph_manager.add_node("test_key", "test_type")
        assert node is not None
        
        retrieved = self.graph_manager.get_node("test_key")
        assert retrieved is not None
        assert retrieved.key == "test_key"
        assert retrieved.type == "test_type"
    
    def test_remove_node(self):
        """Test removing nodes."""
        self.graph_manager.add_node("test_key")
        
        # Verify node exists
        assert self.graph_manager.get_node("test_key") is not None
        
        # Remove node
        removed = self.graph_manager.remove_node("test_key")
        assert removed is True
        
        # Verify node is gone
        assert self.graph_manager.get_node("test_key") is None
    
    def test_add_relationship(self):
        """Test adding relationships between nodes."""
        self.graph_manager.add_node("node1")
        self.graph_manager.add_node("node2")
        
        success = self.graph_manager.add_relationship("node1", "connects_to", "node2")
        assert success is True
        
        node1 = self.graph_manager.get_node("node1")
        assert "connects_to" in node1.relationships
        assert "node2" in node1.relationships["connects_to"]
    
    def test_find_nodes_by_type(self):
        """Test finding nodes by type."""
        self.graph_manager.add_node("node1", "type_a")
        self.graph_manager.add_node("node2", "type_b")
        self.graph_manager.add_node("node3", "type_a")
        
        type_a_nodes = self.graph_manager.find_nodes_by_type("type_a")
        assert len(type_a_nodes) == 2
        
        type_b_nodes = self.graph_manager.find_nodes_by_type("type_b")
        assert len(type_b_nodes) == 1
    
    def test_clear_graph(self):
        """Test clearing the entire graph."""
        self.graph_manager.add_node("node1")
        self.graph_manager.add_node("node2")
        
        assert self.graph_manager.size() == 2
        
        self.graph_manager.clear()
        assert self.graph_manager.size() == 0
    
    def test_get_neighbors(self):
        """Test getting neighboring nodes."""
        self.graph_manager.add_node("center")
        self.graph_manager.add_node("neighbor1")
        self.graph_manager.add_node("neighbor2")
        
        self.graph_manager.add_relationship("center", "connected_to", "neighbor1")
        self.graph_manager.add_relationship("center", "connected_to", "neighbor2")
        
        neighbors = self.graph_manager.get_neighbors("center")
        assert len(neighbors) == 2
        
        # Test with specific relationship type
        connected_neighbors = self.graph_manager.get_neighbors("center", "connected_to")
        assert len(connected_neighbors) == 2