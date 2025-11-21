"""
Logs route - provides access to system logs.
"""
from flask import Blueprint, jsonify, request
from typing import Any


logs_bp = Blueprint('logs', __name__)


@logs_bp.route('/logs', methods=['GET'])
def get_logs() -> Any:
    """
    Get recent logs.

    Returns:
        JSON response with recent log entries
    """
    from ..datastore import datastore_instance

    return jsonify(datastore_instance.tail())
