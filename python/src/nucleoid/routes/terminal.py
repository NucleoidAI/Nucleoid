"""
Terminal route - main application entry point with route aggregation.
"""
from flask import Flask, request, jsonify, make_response
from flask_cors import CORS
from typing import Any, Dict, Optional
import json


def create_terminal_app() -> Flask:
    """
    Create and configure the terminal Flask application.

    Returns:
        Configured Flask application
    """
    app = Flask(__name__)
    CORS(app)

    # Import blueprints
    from .graph import graph_bp
    from .logs import logs_bp
    from .metrics import metrics_bp
    from .openapi import openapi_bp

    # Main runtime endpoint
    @app.route('/', methods=['POST'])
    def process_runtime() -> Any:
        """
        Process runtime code execution.

        Expects:
            Content-Type: application/javascript
            Body: JavaScript code to execute

        Returns:
            Execution details
        """
        # Check content type
        if request.content_type != 'application/javascript':
            return make_response('', 415)

        # Get body as text
        code = request.get_data(as_text=True)

        if not code:
            return make_response('', 400)

        # Process the code
        from ..runtime import runtime_instance

        try:
            details = runtime_instance.process(code, options={'details': True})
            return make_response(details, 200)
        except Exception as e:
            return make_response(str(e), 500)

    # Register blueprints
    app.register_blueprint(graph_bp)
    app.register_blueprint(openapi_bp)
    app.register_blueprint(logs_bp)
    app.register_blueprint(metrics_bp)

    # Error handlers
    @app.errorhandler(400)
    def bad_request(error: Exception) -> Any:
        """Handle bad request errors."""
        if isinstance(error, str):
            return jsonify({'error': error}), 400
        return jsonify({'error': str(error)}), 400

    @app.errorhandler(422)
    def unprocessable_entity(error: Exception) -> Any:
        """Handle JSON parsing errors."""
        return make_response('', 422)

    @app.errorhandler(500)
    def internal_error(error: Exception) -> Any:
        """Handle internal server errors."""
        if isinstance(error, dict) and 'error' in error:
            return jsonify(error), 400
        return make_response(str(error), 500)

    # General error handler
    @app.errorhandler(Exception)
    def handle_exception(error: Exception) -> Any:
        """Handle all unhandled exceptions."""
        if isinstance(error, str):
            return jsonify({'error': error}), 400

        if hasattr(error, 'message'):
            return jsonify({'message': error.message}), 400

        if isinstance(error, dict) and 'error' in error:
            return jsonify(error), 400

        return make_response(str(error), 500)

    return app


# Create the terminal app instance
terminal = create_terminal_app()
