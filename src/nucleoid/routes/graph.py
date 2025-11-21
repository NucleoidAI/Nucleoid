"""
Graph route - provides graph visualization and inspection endpoints.
"""
from flask import Blueprint, jsonify, request
from typing import Dict, Any, List


graph_bp = Blueprint('graph', __name__)


@graph_bp.route('/', methods=['GET'])
def get_graph() -> Any:
    """
    Get the current graph state.

    Returns:
        JSON response with graph nodes and their relationships
    """
    from ..lib.graph import graph as graph_instance

    result: Dict[str, Any] = {}

    for key in graph_instance:
        node = graph_instance[key]

        if hasattr(node, 'key') and node.key:
            tmp: Dict[str, Any] = {
                'id': node.key,
                'type': node.__class__.__name__,
            }

            # Process node properties
            for prop in dir(node):
                if prop.startswith('_'):
                    continue

                try:
                    obj = getattr(node, prop)

                    # Handle single object with key
                    if hasattr(obj, 'key') and obj.key:
                        tmp[prop] = obj.key

                    # Handle dict of objects
                    elif isinstance(obj, dict) and obj:
                        obj_list: List[str] = []

                        for inner_prop in obj:
                            inner_object = obj[inner_prop]

                            if hasattr(inner_object, 'key') and inner_object.key:
                                obj_list.append(inner_object.key)

                        if obj_list:
                            tmp[prop] = obj_list

                except (AttributeError, TypeError):
                    # Skip properties that can't be accessed
                    continue

            result[key] = tmp

    return jsonify(result)
