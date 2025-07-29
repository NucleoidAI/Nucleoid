"""Graph API routes for Nucleoid AI."""

from typing import Dict, Any, List, Union
from fastapi import APIRouter, Request, Response

from ..graph import GraphManager

router = APIRouter()
graph_manager = GraphManager()


@router.get("/")
async def get_graph(request: Request) -> Dict[str, Any]:
    """Get the current knowledge graph."""
    result: Dict[str, Any] = {}
    graph = graph_manager.get_graph()
    
    for key, node in graph.items():
        if hasattr(node, 'key') and node.key:
            node_data = {
                "id": node.key,
                "type": type(node).__name__,
            }
            
            # Process node properties
            for prop_name, prop_value in vars(node).items():
                if isinstance(prop_value, dict) and hasattr(prop_value, 'key'):
                    node_data[prop_name] = prop_value.key
                elif isinstance(prop_value, dict) and not hasattr(prop_value, 'key'):
                    # Handle nested objects
                    prop_list = []
                    for inner_key, inner_value in prop_value.items():
                        if (isinstance(inner_value, dict) and 
                            hasattr(inner_value, 'key')):
                            prop_list.append(inner_value.key)
                    
                    if prop_list:
                        node_data[prop_name] = prop_list
            
            result[key] = node_data
    
    return result