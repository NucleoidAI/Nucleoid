"""
Main FastAPI application for Nucleoid runtime.

This module creates and configures the FastAPI application with all routes,
middleware, and error handling for the Nucleoid runtime API.
"""

import logging

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from .routes.terminal import router as terminal_router

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Create FastAPI application
app = FastAPI(
    title="Nucleoid Runtime API",
    description="Declarative, logic-based, contextual runtime for Neuro-Symbolic AI",
    version="0.7.10",
    docs_url="/docs",
    redoc_url="/redoc"
)

# Add CORS middleware (equivalent to Express cors())
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure as needed for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include terminal router (which includes all sub-routes)
app.include_router(terminal_router)


@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """
    Global exception handler for the FastAPI application.
    
    Args:
        request: The request that caused the exception
        exc: The exception that was raised
        
    Returns:
        JSON error response
    """
    logger.exception("Unhandled exception")

    return JSONResponse(
        status_code=500,
        content={"error": "Internal server error", "detail": str(exc)}
    )


@app.get("/")
async def root():
    """Root endpoint providing basic API information."""
    return {
        "name": "Nucleoid Runtime API",
        "version": "0.7.10",
        "description": "Declarative, logic-based, contextual runtime for Neuro-Symbolic AI"
    }


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
