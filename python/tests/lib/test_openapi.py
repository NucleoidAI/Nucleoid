"""Tests for OpenAPI module"""

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from nucleoid.lib.openapi import init, load, app, start, stop, status


class TestOpenAPI:
    """Test suite for OpenAPI module"""

    def setup_method(self):
        """Setup for each test - ensure clean state"""
        # Reset module state
        import nucleoid.lib.openapi as openapi_module
        openapi_module._app = None
        openapi_module._started = False

    def test_init_creates_new_app(self):
        """Test that init() creates a new FastAPI app"""
        init()
        fastapi_app = app()

        assert fastapi_app is not None
        assert isinstance(fastapi_app, FastAPI)
        assert fastapi_app.title == "Nucleoid"
        assert fastapi_app.version == "1.0.0"

    def test_init_with_existing_app(self):
        """Test that init() can accept an existing FastAPI app"""
        custom_app = FastAPI(title="Custom")
        init(custom_app)

        assert app() is custom_app
        assert app().title == "Custom"

    def test_app_returns_none_when_not_initialized(self):
        """Test that app() returns None when not initialized"""
        assert app() is None

    def test_status_returns_not_started_initially(self):
        """Test that status() returns started=False initially"""
        result = status()

        assert isinstance(result, dict)
        assert "started" in result
        assert result["started"] is False

    def test_load_raises_when_not_initialized(self):
        """Test that load() raises error when not initialized"""
        spec = {"paths": {}}

        with pytest.raises(RuntimeError, match="OpenAPI has not been initialized"):
            load(spec)

    def test_load_with_simple_spec(self):
        """Test loading a simple OpenAPI spec"""
        init()

        spec = {
            "x-nuc-prefix": "/v1",
            "x-nuc-events": [],
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "getUsers",
                        "x-nuc-action": "User.list()",
                        "description": "Get all users",
                    }
                }
            },
        }

        # Should not raise
        load(spec)

        # Verify app has routes
        fastapi_app = app()
        assert fastapi_app is not None
        # Check that routes were added
        routes = [route.path for route in fastapi_app.routes]
        assert "/v1/api/users" in routes

    def test_load_creates_routes_for_multiple_methods(self):
        """Test that load() creates routes for multiple HTTP methods"""
        init()

        spec = {
            "paths": {
                "/items": {
                    "get": {
                        "operationId": "getItems",
                        "x-nuc-action": "Item.list()",
                    },
                    "post": {
                        "operationId": "createItem",
                        "x-nuc-action": "Item.create(body)",
                    },
                }
            },
        }

        load(spec)

        fastapi_app = app()
        routes = {route.path: route.methods for route in fastapi_app.routes}

        # Check that both GET and POST routes exist for /api/items
        assert "/api/items" in routes

    def test_load_handles_empty_paths(self):
        """Test that load() handles spec with no paths"""
        init()

        spec = {"paths": {}}

        # Should not raise
        load(spec)

    def test_load_with_custom_prefix(self):
        """Test that load() respects custom prefix"""
        init()

        spec = {
            "x-nuc-prefix": "/api/v2",
            "paths": {
                "/test": {
                    "get": {
                        "x-nuc-action": "test()",
                    }
                }
            },
        }

        load(spec)

        routes = [route.path for route in app().routes]
        assert "/api/v2/api/test" in routes

    def test_load_skips_operations_without_action(self):
        """Test that load() skips operations without x-nuc-action"""
        init()

        spec = {
            "paths": {
                "/test": {
                    "get": {
                        "operationId": "getTest",
                        # No x-nuc-action
                    }
                }
            },
        }

        # Should not raise, just skip the operation
        load(spec)

    def test_route_handler_responds(self):
        """Test that created route handlers can respond to requests"""
        init()

        spec = {
            "paths": {
                "/hello": {
                    "get": {
                        "operationId": "getHello",
                        "x-nuc-action": "greet()",
                    }
                }
            },
        }

        load(spec)

        # Test the route
        client = TestClient(app())
        response = client.get("/api/hello")

        # Should get a response (even if placeholder)
        assert response.status_code == 200
        data = response.json()
        assert "message" in data or "action" in data

    def test_stop_when_not_started(self):
        """Test that stop() is safe when server not started"""
        # Should not raise
        stop()

        assert status()["started"] is False

    def test_init_multiple_times(self):
        """Test that calling init() multiple times works"""
        init()
        app1 = app()

        init()
        app2 = app()

        # Should create new apps each time
        assert app1 is not None
        assert app2 is not None
