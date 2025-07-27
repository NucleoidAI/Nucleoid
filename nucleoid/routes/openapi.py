"""
OpenAPI routes for Nucleoid runtime.

This module provides endpoints for OpenAPI specification management
and function/declaration loading.
"""

from typing import Any

from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from .. import context
from ..lib import openapi

router = APIRouter()


class FunctionDefinition(BaseModel):
    """Function definition model."""
    definition: str


class DeclarationDefinition(BaseModel):
    """Declaration definition model."""
    definition: str


class OpenAPIAction(BaseModel):
    """OpenAPI action request model."""
    x_nuc_action: str = Field(..., alias="x-nuc-action")
    x_nuc_functions: list[FunctionDefinition] | None = Field(
        default_factory=list, alias="x-nuc-functions"
    )
    x_nuc_declarations: list[DeclarationDefinition] | None = Field(
        default_factory=list, alias="x-nuc-declarations"
    )
    x_nuc_port: int | None = Field(default=None, alias="x-nuc-port")


@router.get("/openapi")
async def get_openapi_status() -> dict[str, Any]:
    """
    Get OpenAPI status information.
    
    Returns the current status of the OpenAPI system including
    loaded specifications and configuration.
    
    Returns:
        OpenAPI status information
    """
    return openapi.status()


@router.post("/openapi")
async def post_openapi_action(action_data: OpenAPIAction) -> None:
    """
    Execute OpenAPI actions.
    
    Handles loading of functions and declarations based on the specified action.
    Currently supports the 'start' action for initializing the OpenAPI system.
    
    Args:
        action_data: The action data including functions and declarations
        
    Raises:
        HTTPException: 400 if action is not supported
    """
    if action_data.x_nuc_action == "start":
        # Load functions
        if action_data.x_nuc_functions:
            function_defs = [
                {"definition": func.definition}
                for func in action_data.x_nuc_functions
            ]
            context.run(function_defs)

        # Load declarations
        if action_data.x_nuc_declarations:
            declaration_defs = [
                {
                    "definition": decl.definition,
                    "options": {"declarative": True}
                }
                for decl in action_data.x_nuc_declarations
            ]
            context.run(declaration_defs)

        # Initialize and load OpenAPI
        openapi.init()
        openapi.load(action_data.dict(by_alias=True))

    else:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Unsupported action: {action_data.x_nuc_action}"
        )
