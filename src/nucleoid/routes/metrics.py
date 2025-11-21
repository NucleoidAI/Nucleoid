"""
Metrics route - provides system metrics and resource usage information.
"""
from flask import Blueprint, jsonify, request
import psutil
from typing import Any


metrics_bp = Blueprint('metrics', __name__)


@metrics_bp.route('/metrics', methods=['GET'])
def get_metrics() -> Any:
    """
    Get system metrics including memory usage.

    Returns:
        JSON response with free and total memory
    """
    memory = psutil.virtual_memory()

    return jsonify({
        'free': memory.available,
        'total': memory.total,
    })
