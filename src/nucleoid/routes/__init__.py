"""
routes module - Flask routes and application endpoints.
"""

from .graph import graph_bp
from .logs import logs_bp
from .metrics import metrics_bp
from .openapi import openapi_bp
from .terminal import terminal, create_terminal_app

__all__ = [
    'graph_bp',
    'logs_bp',
    'metrics_bp',
    'openapi_bp',
    'terminal',
    'create_terminal_app',
]
