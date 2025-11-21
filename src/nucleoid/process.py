"""
Process module for initializing the Nucleoid runtime.
"""
import os
import json
from typing import Any, Dict, Optional


initialized = False


def init() -> None:
    """
    Initialize the Nucleoid runtime process.

    This function:
    1. Initializes the datastore
    2. Loads context from datastore
    3. Loads configuration files
    4. Sets up routes and extensions
    5. Starts the terminal server
    """
    global initialized

    if initialized:
        return

    # Lazy imports to avoid circular dependencies
    from . import datastore
    from . import config
    from . import stack
    from . import context
    from .lib import openapi
    from .routes import terminal

    print("[✓] Data Store is initiated")

    _config = config.config_instance

    datastore.init(_config.to_dict())

    # Load context from datastore
    def load_context() -> None:
        """Load and process stored data."""
        data_list = datastore.read()

        for data in data_list:
            dollar = data.get('$')
            c = data.get('c')
            e = data.get('e')

            if not e and dollar:
                stack.stack_instance.process(
                    dollar,
                    None,
                    type('Options', (), {'declarative': c})()
                )

        print("[✓] Context is loaded")

        # Load nucleoid.js if it exists
        nucleoid_path = os.path.join(_config.path, 'nucleoid.py')
        if os.path.exists(nucleoid_path):
            try:
                import importlib.util
                spec = importlib.util.spec_from_file_location("nucleoid_config", nucleoid_path)
                if spec and spec.loader:
                    module = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(module)
                    if hasattr(module, 'init'):
                        from . import nucleoid as nuc_module
                        module.init(nuc_module)
            except Exception as err:
                print(f"Warning: Failed to load nucleoid.py: {err}")

        # Try to load event extension
        event_path = os.path.join(_config.path, 'extensions', 'event.py')
        if os.path.exists(event_path):
            try:
                import importlib.util
                spec = importlib.util.spec_from_file_location("event_ext", event_path)
                if spec and spec.loader:
                    module = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(module)
                    if hasattr(module, 'init'):
                        module.init()
            except Exception:
                pass  # Silently ignore if event extension doesn't exist

        # Load native routes if configured
        native = _config.options.get('native')
        if native and 'routes' in native:
            try:
                for route in native['routes']:
                    route_path = os.path.join(_config.path, 'native', f"{route}.py")
                    if os.path.exists(route_path):
                        import importlib.util
                        spec = importlib.util.spec_from_file_location(f"native_{route}", route_path)
                        if spec and spec.loader:
                            module = importlib.util.module_from_spec(spec)
                            spec.loader.exec_module(module)
                            # Register the blueprint if it exists
                            if hasattr(module, 'bp'):
                                terminal.terminal.register_blueprint(module.bp)
            except Exception as err:
                print(f"Failed to load native routes: {err}")
                raise

        # Try to load OpenAPI configuration
        openapi_path = os.path.join(_config.path, 'openapi.json')
        if os.path.exists(openapi_path):
            try:
                with open(openapi_path, 'r') as f:
                    openapi_config = json.load(f)

                functions = openapi_config.get('functions')
                if functions:
                    context.run(functions)

                openapi.openapi_instance.init()
                openapi.openapi_instance.load({
                    'api': openapi_config.get('api'),
                    'types': openapi_config.get('types'),
                    'prefix': openapi_config.get('prefix'),
                    'events': openapi_config.get('events'),
                })

                port = openapi_config.get('port', _config.port.get('openapi', 3000))
                openapi.openapi_instance.start(port)

            except FileNotFoundError:
                pass  # OpenAPI config is optional
            except Exception as err:
                print(f"Error loading OpenAPI: {err}")
                import sys
                sys.exit(1)

        print("[✓] Process is running")

        # Start terminal server if not in test mode
        test_mode = _config.options.get('test', False)
        if not test_mode:
            # Add 404 handler
            @terminal.terminal.route('*')
            def catch_all():
                from flask import make_response
                return make_response('', 404)

            # Start the Flask server
            port = _config.port.get('terminal', 8448)
            terminal.terminal.run(host='0.0.0.0', port=port)
            print("[✓] Terminal is ready")

    # Schedule context loading (Python equivalent of setImmediate)
    import threading
    thread = threading.Thread(target=load_context)
    thread.daemon = True
    thread.start()

    initialized = True
