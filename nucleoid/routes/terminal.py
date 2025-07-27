"""
Terminal API routes for Nucleoid runtime.

This module provides the main terminal endpoint for executing Nucleoid statements
and integrates with other route modules for comprehensive API functionality.
"""

import logging
from typing import Any

from fastapi import APIRouter, HTTPException, Request, status
from fastapi.responses import JSONResponse

from .. import runtime
from .graph import router as graph_router
from .logs import router as logs_router
from .metrics import router as metrics_router
from .openapi import router as openapi_router

logger = logging.getLogger(__name__)

# Create the main terminal router
router = APIRouter()

# Include sub-routers
router.include_router(graph_router)
router.include_router(openapi_router)
router.include_router(logs_router)
router.include_router(metrics_router)


@router.post("/")
async def execute_statement(request: Request) -> Any:
    """
    Execute a Nucleoid statement.
    
    Accepts JavaScript code as text input and executes it in the Nucleoid runtime,
    returning detailed execution results.
    
    Args:
        request: FastAPI request object containing JavaScript code
        
    Returns:
        Execution details from the runtime
        
    Raises:
        HTTPException: 415 if content type is not application/javascript
        HTTPException: 422 if request body is invalid
        HTTPException: 400 for validation or execution errors
        HTTPException: 500 for internal server errors
    """
    # Check content type
    content_type = request.headers.get("content-type", "")
    if content_type != "application/javascript":
        raise HTTPException(
            status_code=status.HTTP_415_UNSUPPORTED_MEDIA_TYPE,
            detail="Content-Type must be application/javascript"
        )

    try:
        # Get request body as text
        body = await request.body()
        statement = body.decode("utf-8")

        # Process the statement with runtime
        details = runtime.process(statement, {"details": True})

        return details

    except UnicodeDecodeError:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Invalid UTF-8 content"
        )
    except Exception as e:
        logger.exception("Error processing statement")

        # Handle different error types
        if hasattr(e, 'message'):
            # Validation-like errors
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail={"message": e.message}
            )
        elif hasattr(e, 'error'):
            # Structured errors
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=e.error
            )
        else:
            # General errors
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail=str(e)
            )


# Exception handlers for the router
@router.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """
    Handle general exceptions in the terminal routes.
    
    Args:
        request: The request that caused the exception
        exc: The exception that was raised
        
    Returns:
        JSON error response
    """
    logger.exception("Unhandled exception in terminal route")

    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={"error": str(exc)}
    )
