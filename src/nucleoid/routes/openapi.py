"""
OpenAPI route - handles OpenAPI specification and runtime configuration.
"""
from flask import Blueprint, jsonify, request, make_response
from typing import Any, Dict, List, Optional
import json


openapi_bp = Blueprint('openapi', __name__)


@openapi_bp.route('/openapi', methods=['GET'])
def get_openapi_status() -> Any:
    """
    Get OpenAPI status.

    Returns:
        JSON response with OpenAPI status
    """
    from ..lib.openapi import openapi_instance

    return jsonify(openapi_instance.status())


@openapi_bp.route('/openapi', methods=['POST'])
def post_openapi() -> Any:
    """
    Configure OpenAPI runtime.

    Expected request body:
        {
            "x-nuc-action": "start",
            "x-nuc-functions": [{"definition": "..."}],
            "x-nuc-declarations": [{"definition": "..."}],
            "x-nuc-port": 3000
        }

    Returns:
        Empty response
    """
    from ..lib.openapi import openapi_instance
    from ..context import context_instance

    # Validate request body
    if not request.json:
        return make_response(jsonify({'error': 'Invalid request body'}), 400)

    data: Dict[str, Any] = request.json

    # Extract fields with defaults
    action: str = data.get('x-nuc-action', '')
    functions: List[Dict[str, str]] = data.get('x-nuc-functions', [])
    declarations: List[Dict[str, str]] = data.get('x-nuc-declarations', [])
    port: Optional[int] = data.get('x-nuc-port')

    # Validate required fields
    if not action:
        return make_response(jsonify({'error': 'x-nuc-action is required'}), 400)

    if action == 'start':
        # Process functions
        context_instance.run([
            {'definition': func['definition']}
            for func in functions
        ])

        # Process declarations
        context_instance.run([
            {
                'definition': decl['definition'],
                'options': {'declarative': True}
            }
            for decl in declarations
        ])

        # Initialize and load OpenAPI
        openapi_instance.init()
        openapi_instance.load(data)

    return make_response('', 200)
