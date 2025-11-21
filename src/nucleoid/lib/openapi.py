"""
OpenAPI/REST API server implementation using FastAPI.
Provides OpenAPI/Swagger integration for creating REST APIs from specifications.
"""

import os
import uuid
from pathlib import Path
from typing import Any, Optional

from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import uvicorn


# Module-level state
_app: Optional[FastAPI] = None
_server: Optional[uvicorn.Server] = None
_started: bool = False
_server_thread: Optional[Any] = None


def init(app: Optional[FastAPI] = None) -> None:
    """
    Initialize the OpenAPI server.

    Args:
        app: Optional FastAPI application instance. If not provided, creates a new one.
    """
    global _app, _started

    if app:
        _app = app
    else:
        if _started:
            stop()

        _app = FastAPI(
            title="Nucleoid",
            version="1.0.0",
            openapi_version="3.0.1"
        )

        # Add CORS middleware
        _app.add_middleware(
            CORSMiddleware,
            allow_origins=["*"],
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )


def load(spec: dict[str, Any]) -> None:
    """
    Load an OpenAPI specification and create routes.

    Args:
        spec: OpenAPI specification dictionary

    Raises:
        RuntimeError: If OpenAPI has not been initialized
    """
    global _app

    if _app is None:
        raise RuntimeError("OpenAPI has not been initialized")

    # Extract custom extensions
    prefix = spec.get("x-nuc-prefix", "")
    events = spec.get("x-nuc-events", [])

    # Get paths from spec
    paths = spec.get("paths", {})

    # Create routes for each path
    for path, methods in paths.items():
        for method, operation in methods.items():
            if isinstance(operation, dict):
                _create_route(_app, path, method, operation, prefix)

    # TODO: Handle event extensions
    # This would require integration with the event system


def _create_route(
    app: FastAPI,
    path: str,
    method: str,
    operation: dict[str, Any],
    prefix: str
) -> None:
    """
    Create a route handler for a specific path and method.

    Args:
        app: FastAPI application instance
        path: API path
        method: HTTP method (get, post, put, delete, etc.)
        operation: OpenAPI operation object
        prefix: URL prefix for routes
    """
    # Get the action from the custom extension
    action = operation.get("x-nuc-action")

    if not action:
        return

    # Convert OpenAPI path parameters to FastAPI format
    # e.g., /users/{id} stays as /users/{id}
    route_path = f"{prefix}/api{path}"

    # Create the route handler
    async def route_handler(request: Request) -> JSONResponse:
        """Dynamic route handler for OpenAPI endpoints"""
        # Extract request data
        scope = {
            "params": request.path_params,
            "query": dict(request.query_params),
            "body": await request.json() if await request.body() else {},
            "user": request.headers.get("x-nuc-user"),
        }

        # TODO: Integrate with nucleoid.run() when available
        # For now, return a placeholder response
        result = {"message": "Not implemented", "action": action, "scope": scope}
        error = None

        if not result:
            status_code = 404 if method == "get" else 200
            return Response(status_code=status_code)
        elif error:
            return JSONResponse(status_code=400, content={"error": result})
        else:
            return JSONResponse(status_code=200, content=result)

    # Register the route with FastAPI
    route_methods = [method.upper()]
    app.add_api_route(
        route_path,
        route_handler,
        methods=route_methods,
        name=operation.get("operationId", f"{method}_{path}"),
        description=operation.get("description", ""),
        tags=operation.get("tags", []),
    )


def app() -> Optional[FastAPI]:
    """
    Get the FastAPI application instance.

    Returns:
        The FastAPI application instance or None if not initialized
    """
    return _app


def start(port: int = 3000) -> None:
    """
    Start the OpenAPI server.

    Args:
        port: Port number to listen on (default: 3000)
    """
    global _app, _server, _started, _server_thread

    if _started:
        stop()

    if _app is None:
        raise RuntimeError("OpenAPI has not been initialized")

    # Create uvicorn server
    config = uvicorn.Config(
        _app,
        host="0.0.0.0",
        port=port,
        log_level="info",
    )
    _server = uvicorn.Server(config)

    # Run in a separate thread to avoid blocking
    import threading
    _server_thread = threading.Thread(target=_server.run, daemon=True)
    _server_thread.start()
    _started = True


def stop() -> None:
    """Stop the OpenAPI server if it's running."""
    global _server, _started, _server_thread

    if not _started:
        return

    if _server:
        _server.should_exit = True

    if _server_thread:
        _server_thread.join(timeout=5)

    _started = False


def status() -> dict[str, bool]:
    """
    Get the server status.

    Returns:
        Dictionary with 'started' key indicating if server is running
    """
    return {"started": _started}
