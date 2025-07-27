"""
Graph API routes for Nucleoid runtime.

This module provides endpoints for querying and inspecting the Nucleoid graph
structure, including nodes and their relationships.
"""

from fastapi import APIRouter

from ..graph import graph_store

router = APIRouter()


@router.get("/")
async def get_graph() -> dict[str, dict[str, str | list[str]]]:
    """
    Get the current graph structure.
    
    Returns a JSON representation of all nodes in the graph with their
    relationships and properties. Each node includes its ID, type, and
    connected nodes.
    
    Returns:
        Dictionary mapping node keys to their graph representation
    """
    result: dict[str, dict[str, str | list[str]]] = {}

    for key, node in graph_store.items():
        if hasattr(node, 'key') and node.key:
            # Create base node representation
            node_data = {
                "id": node.key,
                "type": node.__class__.__name__,
            }

            # Process node properties
            for prop_name in dir(node):
                # Skip private attributes and methods
                if prop_name.startswith('_') or callable(getattr(node, prop_name)):
                    continue

                try:
                    prop_value = getattr(node, prop_name)

                    # Handle object properties with keys (single references)
                    if (isinstance(prop_value, object) and
                        hasattr(prop_value, 'key') and
                        prop_value.key):
                        node_data[prop_name] = prop_value.key

                    # Handle dictionary/object properties (multiple references)
                    elif isinstance(prop_value, dict):
                        key_list = []

                        for inner_key, inner_value in prop_value.items():
                            if (isinstance(inner_value, object) and
                                hasattr(inner_value, 'key') and
                                inner_value.key):
                                key_list.append(inner_value.key)

                        if key_list:
                            node_data[prop_name] = key_list

                except (AttributeError, TypeError):
                    # Skip properties that can't be accessed or processed
                    continue

            result[key] = node_data

    return result
