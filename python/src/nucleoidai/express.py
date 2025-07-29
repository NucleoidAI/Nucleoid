"""FastAPI integration for Nucleoid AI."""

import json
from pathlib import Path
from typing import Any, Dict, Optional, Callable, Union

from fastapi import FastAPI, Request, Response, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
import uvicorn

from .context import ContextManager
from .lib.openapi import OpenAPIManager


class NucleoidApp:
    """Nucleoid FastAPI application wrapper."""
    
    def __init__(self, options: Optional[Dict[str, Any]] = None):
        self.app = FastAPI()
        self.options = options or {}
        self.context_manager = ContextManager()
        self.openapi_manager = OpenAPIManager()
        
        # Add CORS middleware
        self.app.add_middleware(
            CORSMiddleware,
            allow_origins=["*"],
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )
        
        # Add JSON middleware equivalent
        self._setup_middleware()
    
    def _setup_middleware(self):
        """Set up middleware similar to Express."""
        pass  # FastAPI handles JSON parsing automatically
    
    def fastapi(self) -> FastAPI:
        """Get the underlying FastAPI instance."""
        return self.app
    
    def address(self) -> Optional[Dict[str, Any]]:
        """Get server address information."""
        # This would be set when the server is running
        return getattr(self, '_server_info', None)
    
    def use(self, middleware: Callable) -> None:
        """Add middleware to the application."""
        self.app.middleware("http")(middleware)
    
    def static(self, mount_path: str, directory: str) -> None:
        """Mount static files."""
        self.app.mount(mount_path, StaticFiles(directory=directory), name="static")
    
    def get(self, path: str, handler: Callable) -> None:
        """Add GET route."""
        @self.app.get(path)
        async def route_handler(request: Request):
            return await self._accept_request(request, handler)
    
    def post(self, path: str, handler: Callable) -> None:
        """Add POST route."""
        @self.app.post(path)
        async def route_handler(request: Request):
            return await self._accept_request(request, handler)
    
    def put(self, path: str, handler: Callable) -> None:
        """Add PUT route."""
        @self.app.put(path)
        async def route_handler(request: Request):
            return await self._accept_request(request, handler)
    
    def delete(self, path: str, handler: Callable) -> None:
        """Add DELETE route."""
        @self.app.delete(path)
        async def route_handler(request: Request):
            return await self._accept_request(request, handler)
    
    async def _accept_request(self, request: Request, handler: Callable) -> Any:
        """Process request similar to Express accept function."""
        from . import nucleoid
        
        # Extract request data
        try:
            body = await request.json() if request.headers.get("content-type", "").startswith("application/json") else {}
        except:
            body = {}
        
        scope = {
            "params": request.path_params,
            "query": dict(request.query_params),
            "body": body
        }
        
        try:
            # Run the handler through nucleoid
            result = nucleoid.run(handler, {"scope": scope, "details": True})
            
            if not result:
                raise HTTPException(status_code=404, detail="Not found")
            elif hasattr(result, 'error') and result.error:
                raise HTTPException(status_code=400, detail=str(result))
            else:
                return result
                
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))
    
    def listen(self, port: int = 3000, callback: Optional[Callable] = None) -> None:
        """Start the server."""
        from . import nucleoid
        
        # Add 404 handler
        @self.app.exception_handler(404)
        async def not_found_handler(request: Request, exc: HTTPException):
            return Response(status_code=404)
        
        # Add 500 error handler
        @self.app.exception_handler(500)
        async def server_error_handler(request: Request, exc: HTTPException):
            return Response(content=str(exc.detail), status_code=500)
        
        # Start nucleoid runtime
        nucleoid.start(self.options)
        
        # Store server info
        self._server_info = {"port": port, "host": "0.0.0.0"}
        
        if callback:
            callback()
        
        # Run the server
        uvicorn.run(self.app, host="0.0.0.0", port=port)
    
    def context(self, path: str) -> None:
        """Load context from file."""
        file_path = Path(path)
        if not file_path.exists():
            raise FileNotFoundError(f"Context file not found: {path}")
        
        with open(file_path, 'r') as f:
            data = json.load(f)
        
        # Convert to context items with declarative flag
        context_items = [
            {**item, "options": {"declarative": True}}
            for item in data
        ]
        
        self.context_manager.load(context_items)
    
    def openapi(self, path: str) -> None:
        """Load OpenAPI specification."""
        try:
            file_path = Path(path)
            if not file_path.exists():
                raise FileNotFoundError(f"OpenAPI file not found: {path}")
            
            with open(file_path, 'r') as f:
                openapi_spec = json.load(f)
            
            self.openapi_manager.init(self.app)
            self.openapi_manager.load(openapi_spec)
            
        except Exception as e:
            raise Exception(f"Problem occurred while opening OpenAPI: {e}")


def create_app(options: Optional[Dict[str, Any]] = None) -> NucleoidApp:
    """Create a new Nucleoid FastAPI application."""
    return NucleoidApp(options)